mod api;
mod sentry;

pub use api::ApiConfig;
pub use sentry::SentryConfig;

use derive_more::Display;
use error_stack::{Result, ResultExt};
use twilight_model::id::{marker::ApplicationMarker, Id};

use crate::{Environment, Sensitive};

#[derive(Debug)]
pub struct Config {
    api: ApiConfig,
    application_id: Option<Id<ApplicationMarker>>,
    environment: Environment,
    token: Sensitive<String>,
    workers: usize,
}

#[derive(Debug, Display)]
#[display(fmt = "Could not load bot configuration")]
pub struct BaseConfigLoadError;
impl error_stack::Context for BaseConfigLoadError {}

impl Config {
    pub fn from_env() -> Result<Self, BaseConfigLoadError> {
        let api = ApiConfig::from_env().change_context(BaseConfigLoadError)?;
        let application_id = memobot_env_vars::var_parsed("MEMOBOT_APPLICATION_ID")
            .change_context(BaseConfigLoadError)?;

        let environment = Environment::from_env().change_context(BaseConfigLoadError)?;
        let token = Self::token_from_env().change_context(BaseConfigLoadError)?;

        let workers = memobot_env_vars::var_parsed("MEMOBOT_WORKERS")
            .change_context(BaseConfigLoadError)?
            .unwrap_or_else(Self::default_workers);

        Ok(Self {
            api,
            application_id,
            environment,
            token: Sensitive::new(token),
            workers,
        })
    }
}

impl Config {
    #[must_use]
    pub fn api(&self) -> &ApiConfig {
        &self.api
    }

    #[must_use]
    pub fn application_id(&self) -> Option<Id<ApplicationMarker>> {
        self.application_id
    }

    #[must_use]
    pub fn environment(&self) -> Environment {
        self.environment
    }

    #[must_use]
    pub fn token(&self) -> &str {
        self.token.as_str()
    }

    #[must_use]
    pub fn workers(&self) -> usize {
        self.workers
    }
}

impl Config {
    fn token_from_env() -> Result<String, memobot_env_vars::ReadVarError> {
        memobot_env_vars::var("DISCORD_TOKEN")
            .transpose()
            .or_else(|| memobot_env_vars::var("TOKEN").transpose())
            .unwrap_or_else(|| memobot_env_vars::required_var("MEMOBOT_TOKEN"))
    }

    #[must_use]
    fn default_workers() -> usize {
        num_cpus::get()
    }
}
