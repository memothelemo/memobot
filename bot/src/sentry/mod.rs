use sentry::ClientInitGuard;
use std::borrow::Cow;

use memobot_kernel::config::SentryConfig;
use memobot_kernel::Sensitive;

pub fn init() -> Option<ClientInitGuard> {
    let config = match SentryConfig::from_env() {
        Ok(Some(config)) => config,
        Ok(None) => {
            tracing::info!("Sentry integration is disabled");
            return None;
        }
        Err(error) => {
            tracing::warn!(%error, "Failed to read Sentry configuration from environment");
            return None;
        }
    };

    let opts = sentry::ClientOptions {
        dsn: Some(config.dsn().clone()),
        environment: Some(Cow::Owned(config.environment().to_string())),
        release: sentry::release_name!(),
        session_mode: sentry::SessionMode::Request,
        traces_sample_rate: config.traces_sample_rate(),
        ..Default::default()
    };

    tracing::info!(
        cfg.dsn = ?Sensitive::new(()),
        cfg.environment = %config.environment(),
        opts.release = ?sentry::release_name!(),
        "Sentry integration is enabled"
    );

    Some(sentry::init(opts))
}
