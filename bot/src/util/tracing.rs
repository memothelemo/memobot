use tracing::level_filters::LevelFilter;
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt, Layer};

pub fn init() {
    let targets = dotenvy::var("RUST_LOG")
        .unwrap_or_else(|_| "info".into())
        .trim()
        .trim_matches('"')
        .parse::<Targets>()
        .expect("Failed to parse `RUST_LOG` parameters");

    let log_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .without_time()
        .with_filter(targets);

    let sentry_layer = sentry::integrations::tracing::layer().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry()
        .with(log_layer)
        .with(sentry_layer)
        .init();
}
