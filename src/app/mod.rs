use anyhow::{Context, Result};
use futures::TryFutureExt;
use std::fmt::Debug;
use std::future::IntoFuture;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use twilight_model::id::{marker::ApplicationMarker, Id};

use crate::config::Config;

mod shutdown;
pub use shutdown::ShutdownReason;

#[derive(Clone)]
pub struct App {
    // Finalized application id done from http request or gateway
    application_id: Arc<RwLock<Id<ApplicationMarker>>>,
    config: Arc<Config>,
    http: Arc<twilight_http::Client>,
    shutdown: CancellationToken,
}

impl App {
    pub async fn new(config: Config) -> Result<Self> {
        let http = twilight_http::Client::builder()
            .token(config.token().into())
            .build();

        let application_id = Self::get_application_id(&config, &http).await?;

        Ok(Self {
            application_id: Arc::new(RwLock::new(application_id)),
            config: Arc::new(config),
            http: Arc::new(http),
            shutdown: CancellationToken::new(),
        })
    }

    async fn get_application_id(
        config: &Config,
        http: &twilight_http::Client,
    ) -> Result<Id<ApplicationMarker>> {
        if let Some(id) = config.application_id() {
            return Ok(id);
        }

        tracing::info!("ROBOT_BOT_APPLICATION_ID is missing, getting application ID from Discord");
        http.current_user_application()
            .into_future()
            .map_err(anyhow::Error::new)
            .and_then(|v| v.model().map_err(anyhow::Error::new))
            .map_ok(|v| v.id)
            .await
            .context("Failed to get user application info")
    }
}

impl App {
    #[must_use]
    pub async fn application_id(&self) -> Id<ApplicationMarker> {
        *self.application_id.read().await
    }

    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    #[must_use]
    pub fn http(&self) -> &twilight_http::Client {
        &self.http
    }

    #[must_use]
    pub async fn interaction(&self) -> twilight_http::client::InteractionClient<'_> {
        self.http.interaction(*self.application_id.read().await)
    }
}

impl App {
    pub(crate) async fn override_application_id(&self, new: Id<ApplicationMarker>) {
        *self.application_id.write().await = new;
    }
}

impl Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("application_id", &self.application_id)
            .field("config", &self.config)
            .field("is_shutdown", &self.is_shutdown())
            .finish()
    }
}
