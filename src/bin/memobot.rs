use anyhow::{Context, Result};
use tokio::task::JoinSet;

use memobot::app::{App, ShutdownReason};
use memobot::config::Config;

#[tracing::instrument(skip(app))]
async fn init_shards(app: &App) -> Result<Vec<twilight_gateway::Shard>> {
    use twilight_gateway::stream::create_recommended;
    use twilight_gateway::Intents;

    let intents = Intents::GUILDS | Intents::GUILD_INTEGRATIONS;

    let primary_config = twilight_gateway::Config::new(app.config().token().into(), intents);
    let shards = create_recommended(app.http(), primary_config, |_, builder| builder.build())
        .await
        .context("Failed to initialize shards")?
        .collect();

    Ok(shards)
}

fn main() -> Result<()> {
    memobot::util::tracing::init();

    let config = Config::from_env().unwrap();
    let _sentry = memobot::sentry::init_sentry();
    tracing::info!("Running memobot with {} thread(s)", config.workers());

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(config.workers())
        .build()
        .context("Failed to initialize async runtime")?;

    let app = rt.block_on(App::new(config))?;
    let shards = rt.block_on(init_shards(&app))?;

    rt.block_on(async move {
        let mut services = JoinSet::new();
        services.spawn(memobot::services::bot::start(app.clone(), shards));
        services.spawn(memobot::services::paradise::start(app.clone()));

        let shutdown_signal = memobot::util::shutdown_signal();
        tokio::select! {
            _ = app.shutdown_guard() => {},
            _ = shutdown_signal => {
                app.shutdown(ShutdownReason::Signal);
            },
        };

        while services.join_next().await.is_some() {}
    });

    tracing::info!("All services has been gracefully shutdown. Closing application...");
    Ok(())
}
