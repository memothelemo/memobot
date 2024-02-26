use crate::util::Sensitive;

use anyhow::Result;
use memobot_env_vars::var_parsed;
use sentry::types::Dsn;

#[derive(Debug)]
pub struct SentryConfig {
    dsn: Sensitive<Dsn>,
    environment: SentryEnv,
    traces_sample_rate: f32,
}

impl SentryConfig {
    pub fn from_env() -> Result<Option<SentryConfig>> {
        let Some(dsn) = var_parsed::<Dsn>("ROBOT_SENTRY_DSN")? else {
            return Ok(None);
        };

        let environment =
            var_parsed::<SentryEnv>("ROBOT_SENTRY_ENV")?.unwrap_or(SentryEnv::Development);

        let traces_sample_rate = var_parsed("ROBOT_SENTRY_TRACES_SAMPLE_RATE")?
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
    pub fn environment(&self) -> SentryEnv {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SentryEnv {
    Development,
    Production,
}

impl std::fmt::Display for SentryEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SentryEnv::Development => f.write_str("development"),
            SentryEnv::Production => f.write_str("production"),
        }
    }
}

#[derive(Debug)]
pub struct SentryEnvParseError(String);

impl std::fmt::Display for SentryEnvParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown environment for {:?}", self.0)
    }
}

impl std::error::Error for SentryEnvParseError {}

impl std::str::FromStr for SentryEnv {
    type Err = SentryEnvParseError;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Self::Development),
            "production" | "prod" => Ok(Self::Production),
            _ => Err(SentryEnvParseError(s.into())),
        }
    }
}
