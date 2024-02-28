use futures::future::Either;
use memobot_kernel::{Kernel, ShutdownReason};
use tokio_util::task::TaskTracker;
use twilight_gateway::Event;
use twilight_gateway::{error::ReceiveMessageErrorType, CloseFrame, Shard};

use crate::bot::Context;

#[tracing::instrument(skip_all, fields(
    event.guild_id = ?event.guild_id(),
    event.kind = ?event.kind(),
))]
pub async fn process_event(ctx: Context, event: Event) {
    match event {
        Event::Ready(info) => {
            tracing::info!("Logged in as {} ({})", info.user.name, info.user.id);

            // Replace `application_id` if it figures out it is out of match
            let original_app_id = ctx.kernel().application_id().await;
            let new_app_id = info.application.id;

            if new_app_id != original_app_id {
                tracing::warn!(
                    app.application_id = %original_app_id,
                    event.application.id = %new_app_id,
                    "Unmatched application ID, replacing with new ID"
                );
                ctx.kernel().override_application_id(new_app_id).await;
            }
        }
        _ => {}
    }
}

#[tracing::instrument(skip_all, fields(shard.id = %shard.id()))]
pub async fn main(kernel: Kernel, mut shard: Shard) {
    let context = Context::new(&kernel, shard.id());
    let tasks = TaskTracker::new();

    loop {
        let action = next_event(&kernel, &mut shard).await;
        let event = match action {
            ShardAction::Event(e) => e,
            ShardAction::Ignore => continue,
            ShardAction::CloseLoop => break,
        };

        tasks.spawn(process_event(context.clone(), event));
    }

    tracing::info!("Closing all shard tasks");
    tasks.close();
    tasks.wait().await;

    let is_disconnected = shard.status().is_disconnected();
    if !is_disconnected {
        tracing::info!("Disconnecting shard...");
        close_shard(&mut shard).await;
    }
}

async fn close_shard(shard: &mut Shard) {
    if let Err(error) = shard.close(CloseFrame::NORMAL).await {
        tracing::error!(?error, "Failed to close shard connection");
    }

    // Wait until WebSocket connection is FINALLY CLOSED
    loop {
        match shard.next_message().await {
            Ok(..) => break,
            // interesting error while I was hosting my own bot, you can
            // disable this if you really want to.
            Err(source) if matches!(source.kind(), ReceiveMessageErrorType::Io) => {
                break;
            }
            Err(source) => {
                if source.is_fatal() {
                    tracing::error!(?source, "Got fatal shard message error");
                } else {
                    tracing::warn!(?source, "Got shard message error");
                }
            }
        }
    }
}

enum ShardAction {
    Event(twilight_gateway::Event),
    Ignore,
    CloseLoop,
}

async fn next_event(app: &Kernel, shard: &mut Shard) -> ShardAction {
    use futures::future::select;

    let id = shard.id();
    match select(Box::pin(shard.next_event()), Box::pin(app.shutdown_guard())).await {
        Either::Left((Ok(event), _)) => ShardAction::Event(event),
        Either::Left((Err(source), _)) => {
            if source.is_fatal() {
                tracing::error!(?source, "Got fatal shard message error");
                app.shutdown(ShutdownReason::ShardFatalError(id));
                ShardAction::CloseLoop
            } else {
                tracing::warn!(?source, "Got shard message error");
                ShardAction::Ignore
            }
        }
        Either::Right(_) => ShardAction::CloseLoop,
    }
}
