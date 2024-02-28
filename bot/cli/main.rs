use derive_more::Display;
use error_stack::{Result, ResultExt};
use memobot_kernel::{Kernel, ShutdownReason};
use tokio::task::JoinSet;

#[tracing::instrument(skip(kernel))]
async fn init_shards(kernel: &Kernel) -> Result<Vec<twilight_gateway::Shard>, StartError> {
    use twilight_gateway::stream::create_recommended;
    use twilight_gateway::Intents;

    let intents = Intents::GUILDS | Intents::GUILD_INTEGRATIONS;

    let primary_config = twilight_gateway::Config::new(kernel.config().token().into(), intents);
    let shards = create_recommended(kernel.http(), primary_config, |_, builder| builder.build())
        .await
        .change_context(StartError)?
        .collect();

    Ok(shards)
}

#[derive(Debug, Display)]
#[display(fmt = "Failed to start memobot service")]
struct StartError;
impl error_stack::Context for StartError {}

fn main() -> Result<(), StartError> {
    memobot::util::tracing::init();

    let config = memobot_kernel::Config::from_env()
        .change_context(StartError)
        .attach_printable("failed to load configuration")?;

    let _sentry = memobot::sentry::init();
    tracing::info!("Running memobot with {} thread(s)", config.workers());
    tracing::trace!("Initializing tokio runtime");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(config.workers())
        .build()
        .change_context(StartError)
        .attach_printable("could not initialize async runtime")?;

    let kernel = rt
        .block_on(memobot_kernel::Kernel::init(config))
        .change_context(StartError)?;

    let paradise = memobot_paradise::Config::from_env()
        .change_context(StartError)
        .attach_printable("failed to load Paradise configuration")?
        .map(|v| memobot_paradise::Service::new(v, kernel.clone()));

    rt.block_on(async move {
        use actix_web::{web, App, HttpServer};

        let mut services = JoinSet::new();
        let kernel_1 = kernel.clone();
        let api_config = kernel.config().api();

        let http = HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(paradise.clone()))
                .app_data(web::Data::new(kernel_1.clone()))
                .service(web::scope("/paradise").configure(memobot_paradise::api::configure))
        })
        .workers(1)
        .bind((api_config.address(), api_config.port()))
        .change_context(StartError)
        .attach_printable("failed to initialize API server")?
        .run();

        tracing::info!(
            "Listening API server at http://{}:{}",
            api_config.address(),
            api_config.port()
        );

        let kernel_1 = kernel.clone();
        services.spawn(async move {
            let server_handle = http.handle();
            tokio::select! {
                _ = kernel_1.shutdown_guard() => {
                    tracing::info!("Shutting down API server...");
                    server_handle.stop(true).await;
                },
                Err(error) = http => {
                    tracing::error!(?error, "Critical error found in HTTP API server");
                    kernel_1.shutdown(ShutdownReason::ApiServerFailed);
                }
            }
        });

        let shards = init_shards(&kernel).await?;
        services.spawn(memobot::services::bot::start(kernel.clone(), shards));

        let shutdown_signal = memobot::util::shutdown_signal();
        tokio::select! {
            _ = kernel.shutdown_guard() => {},
            _ = shutdown_signal => {
                kernel.shutdown(ShutdownReason::Signal);
            },
        };

        while services.join_next().await.is_some() {}

        tracing::info!("All services has been gracefully shutdown. Closing application...");
        Ok(())
    })
}
