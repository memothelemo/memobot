use anyhow::{anyhow, bail, Context, Result};
use futures::TryFutureExt;
use std::future::IntoFuture;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_util::task::TaskTracker;
use twilight_mention::Mention;

use crate::config::ParadiseConfig;
use crate::{app::App, util::Sensitive};

const SANCTUARY_CHECK_INTERVAL: Duration = Duration::from_secs(120);

// 5 minutes check if aternos is online
const SANCTUARY_CHECK_INTERVAL_OUTAGE: Duration = Duration::from_secs(300);

async fn alert_everyone(app: App, is_online: bool) -> Result<()> {
    tracing::info!(?is_online, "Sending alert message to Paradise");

    let config = app
        .config()
        .paradise()
        .ok_or_else(|| anyhow!("Paradise config is unexpectedly missing"))?;

    let alert_role_mention = config.alert_role_id().mention();
    let mut message = if is_online {
        format!("{alert_role_mention} 🎉 Sanctuary is back online!")
    } else {
        format!("{alert_role_mention} 🎉 Sanctuary is offline")
    };
    message.push_str(&format!(
        "\n\n**Join us at:** `{}:{}`",
        config.sanctuary_addr(),
        config.sanctuary_port()
    ));

    app.http()
        .create_message(config.alert_channel_id())
        .content(&message)
        .context("Failed to validate alert message")?
        .into_future()
        .map_err(anyhow::Error::new)
        .and_then(|v| v.model().map_err(anyhow::Error::new))
        .await
        .context("Failed to send alert message")?;

    Ok(())
}

async fn get_sanctuary_status(config: &ParadiseConfig) -> Result<String> {
    let mut client =
        elytra_ping::connect((config.sanctuary_addr().to_string(), config.sanctuary_port()))
            .await
            .context("Cannot connect to Sanctuary server")?;

    tracing::debug!("Performing server handshake");
    client.handshake().await.map_err(anyhow::Error::new)?;

    tracing::debug!("Sending status request");
    client
        .write_frame(elytra_ping::protocol::Frame::StatusRequest)
        .await
        .map_err(anyhow::Error::new)?;

    tracing::debug!("Reading server protocol frame");
    let frame: elytra_ping::protocol::Frame = client
        .read_frame(None)
        .await?
        .context("Connection closed by server")?;

    let status: String = match frame {
        elytra_ping::protocol::Frame::StatusResponse { json } => json,
        _ => bail!("Unexpected non-status response"),
    };
    tracing::debug!("Got server status");

    Ok(status)
}

#[tracing::instrument(skip(app))]
async fn is_sanctuary_online(app: App) -> Result<bool> {
    const MAX_TRIES: usize = 3;

    tracing::debug!("Checking Sanctuary server status");

    let config = app
        .config()
        .paradise()
        .ok_or_else(|| anyhow!("Paradise config is unexpectedly missing"))?;

    let mut times = 0;
    loop {
        times += 1;

        let status = match get_sanctuary_status(config).await {
            Ok(status) => status,
            Err(error) => {
                if times < MAX_TRIES {
                    tracing::warn!(%error, "Failed to check Sanctuary server status, retrying...");
                    continue;
                }
                return Err(error);
            }
        };

        return Ok(!status.contains("offline"));
    }
}

pub async fn start(app: App) {
    let Some(config) = app.config().paradise() else {
        tracing::info!("Paradise background service is disabled");
        return;
    };

    tracing::info!(
        cfg.id = %config.id(),
        cfg.alert_channel_id = %config.alert_channel_id(),
        cfg.sanctuary_addr = ?Sensitive::new(()),
        cfg.sanctuary_port = ?Sensitive::new(()),
        "Starting Paradise background service"
    );

    let mut interval = tokio::time::interval(SANCTUARY_CHECK_INTERVAL);

    let (outage_tx, mut outage_rx) = mpsc::unbounded_channel::<bool>();
    let previous_value = Arc::new(AtomicBool::new(false));
    let tasks = TaskTracker::new();

    loop {
        tokio::select! {
            _ = interval.tick() => {},
            Some(is_down) = outage_rx.recv() => {
                let new_interval = if is_down {
                    SANCTUARY_CHECK_INTERVAL_OUTAGE
                } else {
                    SANCTUARY_CHECK_INTERVAL
                };

                if interval.period() == new_interval {
                    continue;
                }

                if is_down {
                    tracing::warn!("Aternos is down, triggering outage mode");
                } else {
                    tracing::warn!("Aternos is back, triggering normal mode");
                }

                interval = tokio::time::interval(new_interval);
                continue;
            },
            _ = app.shutdown_guard() => {
                break;
            }
        }

        let app = app.clone();
        let previous_value = previous_value.clone();
        let outage_tx = outage_tx.clone();

        tasks.spawn(async move {
            let is_online = match is_sanctuary_online(app.clone()).await {
                Ok(n) => n,
                Err(error) => {
                    // Inform the loop that there's an outage going on
                    tracing::error!(?error, "Failed to check status of sanctuary server");
                    let _ = outage_tx.send(true);
                    return;
                }
            };

            tracing::debug!(?is_online, "Checking server status done");
            if previous_value.load(Ordering::SeqCst) == is_online {
                return;
            }

            previous_value.store(is_online, Ordering::SeqCst);
            let _ = outage_tx.send(false);

            // Inform everyone who has server ping role to the Paradise server
            // that the server is online! :)
            if let Err(error) = alert_everyone(app, is_online).await {
                tracing::error!(?error, "Failed to alert everyone in Paradise guild");
            }
        });
    }

    tracing::info!("Waiting for {} task(s) to be completed", tasks.len());
    tasks.close();
    tasks.wait().await;

    tracing::info!("Closing Paradise background service...");
}
