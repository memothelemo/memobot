use crate::util::Sensitive;
use anyhow::Result;
use memobot_env_vars::var_parsed;
use twilight_model::id::{marker::ApplicationMarker, Id};

mod paradise;
mod sentry;

pub use paradise::ParadiseConfig;
pub use sentry::SentryConfig;

#[derive(Debug)]
pub struct Config {
    application_id: Option<Id<ApplicationMarker>>,
    paradise: Option<ParadiseConfig>,
    token: Sensitive<String>,
    workers: usize,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let application_id = var_parsed::<Id<ApplicationMarker>>("ROBOT_BOT_APPLICATION_ID")?;

        let token = Self::token_from_env()?;
        let workers = var_parsed("ROBOT_WORKERS")?.unwrap_or_else(Self::default_workers);
        let paradise = ParadiseConfig::from_env()?;

        Ok(Self {
            application_id,
            paradise,
            token: Sensitive::new(token),
            workers,
        })
    }
}

impl Config {
    #[must_use]
    pub fn application_id(&self) -> Option<Id<ApplicationMarker>> {
        self.application_id
    }

    #[must_use]
    pub fn paradise(&self) -> Option<&ParadiseConfig> {
        self.paradise.as_ref()
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
    fn token_from_env() -> Result<String> {
        memobot_env_vars::var("DISCORD_TOKEN")
            .transpose()
            .or_else(|| memobot_env_vars::var("TOKEN").transpose())
            .unwrap_or_else(|| memobot_env_vars::required_var("ROBOT_BOT_TOKEN"))
    }

    #[must_use]
    fn default_workers() -> usize {
        num_cpus::get()
    }
}
