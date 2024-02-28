use derive_more::Display;
use error_stack::{FutureExt, Result, ResultExt};
use futures::TryFutureExt;
use std::future::IntoFuture;
use twilight_mention::Mention;

use crate::Service;

#[derive(Debug, Display)]
#[display(fmt = "Failed to alert everyone on Discord")]
pub struct AlertEveryoneError;
impl error_stack::Context for AlertEveryoneError {}

#[tracing::instrument(skip(service))]
pub async fn alert_everyone(service: &Service, is_online: bool) -> Result<(), AlertEveryoneError> {
    tracing::info!(?is_online, "Sending alert message to Paradise");

    let message;
    let config = service.config();

    if is_online {
        message = format!(
            "🎉  **Sanctuary is back online!** 🎉\nJoin us at: `{}:{}`\n\n{}",
            config.sanctuary_addr(),
            config.sanctuary_port(),
            config.alert_role_id().mention(),
        );
    } else {
        message = format!("❌  **Sanctuary is offline** ❌\nJoin with us next time.");
    }

    let message = service
        .kernel()
        .http()
        .create_message(config.alert_channel_id())
        .content(&message)
        .change_context(AlertEveryoneError)
        .attach_printable("could not validate alert message")?;

    message
        .into_future()
        .change_context(AlertEveryoneError)
        .attach_printable("failed to send message")
        .and_then(|v| {
            v.model()
                .change_context(AlertEveryoneError)
                .attach_printable("failed to parse response body")
        })
        .await?;

    Ok(())
}
