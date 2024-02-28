use cfg_if::cfg_if;
use derive_more::Display;
use error_stack::{FutureExt, Report, Result, ResultExt};
use futures::{Future, TryFutureExt};
use std::fmt::Display;
use std::future::IntoFuture;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tokio_util::task::task_tracker::TaskTrackerWaitFuture;
use tokio_util::task::TaskTracker;
use twilight_model::gateway::ShardId;
use twilight_model::id::{marker::ApplicationMarker, Id};

mod sensitive;
mod suggestion;

pub mod config;

pub use self::config::Config;
pub use self::sensitive::Sensitive;
pub use self::suggestion::Suggestion;

///////////////////////////////////////////////////////////////////////
mod shutdown;

#[derive(Clone)]
pub struct Kernel {
    application_id: Arc<RwLock<Id<ApplicationMarker>>>,
    // Background tasks made from the API to implement things
    // like graceful shutdowns
    background_tasks: TaskTracker,
    config: Arc<config::Config>,
    http: Arc<twilight_http::Client>,
    shutdown: CancellationToken,
}

#[derive(Debug, Display)]
#[display(fmt = "Could not initialize memobot kernel")]
pub struct KernelInitError;
impl error_stack::Context for KernelInitError {}

impl Kernel {
    pub async fn init(config: config::Config) -> Result<Self, KernelInitError> {
        let http = twilight_http::Client::builder()
            .token(config.token().into())
            .build();

        let application_id = Self::get_application_id(&config, &http).await?;

        Ok(Self {
            application_id: Arc::new(RwLock::new(application_id)),
            background_tasks: TaskTracker::new(),
            config: Arc::new(config),
            http: Arc::new(http),
            shutdown: CancellationToken::new(),
        })
    }

    async fn get_application_id(
        config: &config::Config,
        http: &twilight_http::Client,
    ) -> Result<Id<ApplicationMarker>, KernelInitError> {
        if let Some(id) = config.application_id() {
            return Ok(id);
        }

        tracing::warn!("MEMOBOT_APPLICATION_ID is missing, getting application ID from Discord");
        http.current_user_application()
            .into_future()
            .change_context(KernelInitError)
            .and_then(|v| v.model().change_context(KernelInitError))
            .map_ok(|v| v.id)
            .await
            .attach_printable("failed to get application ID of a bot from Discord API")
    }
}

impl Kernel {
    #[must_use]
    pub async fn application_id(&self) -> Id<ApplicationMarker> {
        *self.application_id.read().await
    }

    #[must_use]
    pub fn config(&self) -> &config::Config {
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

impl Kernel {
    #[track_caller]
    #[inline]
    pub fn spawn<F>(&self, task: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.background_tasks.spawn(task)
    }

    #[track_caller]
    #[inline]
    pub fn spawn_blocking<F, T>(&self, task: F) -> JoinHandle<T>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        self.background_tasks.spawn_blocking(task)
    }

    #[must_use]
    pub fn remaining_background_tasks(&self) -> usize {
        self.background_tasks.len()
    }

    pub async fn close_background_tasks_and_wait(&self) -> TaskTrackerWaitFuture<'_> {
        self.background_tasks.close();
        self.background_tasks.wait()
    }

    // #[must_use]
    // #[inline]
    // pub async fn spawn_blocking_result<F, T>(&self, task: F) -> Result<T>
    // where
    //     F: FnOnce() -> Result<T>,
    //     F: Send + 'static,
    //     T: Send + 'static,
    // {
    //     use sentry::Hub;
    //     use std::convert::identity;

    //     let current_span = tracing::Span::current();
    //     let hub = sentry::Hub::current();
    //     self.tasks
    //         .spawn_blocking(move || current_span.in_scope(|| Hub::run(hub, task)))
    //         .await
    //         .map_err(anyhow::Error::new)
    //         .and_then(identity)
    // }
}

impl Kernel {
    #[doc(hidden)]
    pub async fn override_application_id(&self, new: Id<ApplicationMarker>) {
        *self.application_id.write().await = new;
    }
}

impl std::fmt::Debug for Kernel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("application_id", &self.application_id)
            .field("config", &self.config)
            .field("is_shutdown", &self.is_shutdown())
            .finish()
    }
}

///////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShutdownReason {
    ApiServerFailed,
    ShardFatalError(ShardId),
    Signal,
}

impl Display for ShutdownReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ApiServerFailed => f.write_str("API server failed"),
            Self::ShardFatalError(id) => write!(f, "Shard {id} got a fatal error"),
            Self::Signal => f.write_str("Received shutdown signal"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Environment {
    Development,
    Production,
    Testing,
}

impl Environment {
    #[must_use]
    pub const fn from_build() -> Self {
        cfg_if! {
            if #[cfg(release)] {
                Environment::Production
            } else if #[cfg(test)] {
                Environment::Testing
            } else {
                Environment::Development
            }
        }
    }

    #[must_use]
    pub fn from_env() -> Result<Self, memobot_env_vars::ReadVarError> {
        memobot_env_vars::var_parsed::<Environment, _>("MEMOBOT_ENV")
            .map(|v| v.unwrap_or(Environment::from_build()))
    }
}

#[derive(Debug, Display)]
#[display(fmt = "Could not parse environment type")]
pub struct EnvParseError;
impl error_stack::Context for EnvParseError {}

impl Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Development => f.write_str("development"),
            Environment::Production => f.write_str("production"),
            Environment::Testing => f.write_str("testing"),
        }
    }
}

impl std::str::FromStr for Environment {
    type Err = Report<EnvParseError>;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            "testing" => Ok(Self::Testing),
            _ => Err(Report::new(EnvParseError))
                .attach(Suggestion::new(
                    "choose environment type only either 'development' or 'production'",
                ))
                .attach_printable_lazy(|| format!("{s:?} could not be parsed")),
        }
    }
}
