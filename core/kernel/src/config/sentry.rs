use crate::{Environment, Sensitive};

use derive_more::Display;
use error_stack::{Result, ResultExt};
use memobot_env_vars::var_parsed;
use sentry::types::Dsn;

#[derive(Debug)]
pub struct SentryConfig {
    dsn: Sensitive<Dsn>,
    environment: Environment,
    traces_sample_rate: f32,
}

#[derive(Debug, Display)]
#[display(fmt = "Could not load Sentry configuration")]
pub struct SentryConfigLoadError;
impl error_stack::Context for SentryConfigLoadError {}

impl SentryConfig {
    pub fn from_env() -> Result<Option<SentryConfig>, SentryConfigLoadError> {
        let dsn = var_parsed::<Dsn, _>("MEMOBOT_SENTRY_DSN").change_context(SentryConfigLoadError);
        let Some(dsn) = dsn? else {
            return Ok(None);
        };

        let environment = Environment::from_env().change_context(SentryConfigLoadError)?;
        let traces_sample_rate = var_parsed("MEMOBOT_SENTRY_TRACES_SAMPLE_RATE")
            .change_context(SentryConfigLoadError)?
            .unwrap_or(Self::default_traces_sample_rate());

        Ok(Some(Self {
            dsn: Sensitive::new(dsn),
            environment,
            traces_sample_rate,
        }))
    }
}

impl SentryConfig {
    #[must_use]
    pub fn dsn(&self) -> &Dsn {
        self.dsn.as_ref()
    }

    #[must_use]
    pub fn environment(&self) -> Environment {
        self.environment
    }

    #[must_use]
    pub fn traces_sample_rate(&self) -> f32 {
        self.traces_sample_rate
    }
}

impl SentryConfig {
    #[must_use]
    fn default_traces_sample_rate() -> f32 {
        1.
    }
}
