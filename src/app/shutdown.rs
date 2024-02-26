use std::fmt::Display;
use tokio_util::sync::WaitForCancellationFuture;
use twilight_gateway::ShardId;

use super::App;

// impl App {
//     // #[track_caller]
//     // #[inline]
//     // pub fn spawn<F>(&self, task: F) -> JoinHandle<F::Output>
//     // where
//     //     F: Future + Send + 'static,
//     //     F::Output: Send + 'static,
//     // {
//     //     self.tasks.spawn(task)
//     // }

//     // #[track_caller]
//     // #[inline]
//     // pub fn spawn_blocking<F, T>(&self, task: F) -> JoinHandle<T>
//     // where
//     //     F: FnOnce() -> T,
//     //     F: Send + 'static,
//     //     T: Send + 'static,
//     // {
//     //     self.tasks.spawn_blocking(task)
//     // }

//     // #[must_use]
//     // #[inline]
//     // pub async fn spawn_blocking_result<F, T>(&self, task: F) -> Result<T>
//     // where
//     //     F: FnOnce() -> Result<T>,
//     //     F: Send + 'static,
//     //     T: Send + 'static,
//     // {
//     //     use sentry::Hub;
//     //     use std::convert::identity;

//     //     let current_span = tracing::Span::current();
//     //     let hub = sentry::Hub::current();
//     //     self.tasks
//     //         .spawn_blocking(move || current_span.in_scope(|| Hub::run(hub, task)))
//     //         .await
//     //         .map_err(anyhow::Error::new)
//     //         .and_then(identity)
//     // }
// }

impl App {
    #[must_use]
    pub fn is_shutdown(&self) -> bool {
        self.shutdown.is_cancelled()
    }

    pub fn shutdown_guard(&self) -> WaitForCancellationFuture<'_> {
        self.shutdown.cancelled()
    }

    pub fn shutdown(&self, reason: ShutdownReason) {
        if self.is_shutdown() {
            return;
        }

        tracing::warn!("{reason}; performing graceful shutdown...");
        self.shutdown.cancel();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShutdownReason {
    ShardFatalError(ShardId),
    Signal,
}

impl Display for ShutdownReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ShardFatalError(id) => write!(f, "Shard {id} got a fatal error"),
            Self::Signal => f.write_str("Received shutdown signal"),
        }
    }
}

// impl App {
//     #[track_caller]
//     #[inline]
//     pub fn spawn<F>(&self, task: F) -> JoinHandle<F::Output>
//     where
//         F: Future + Send + 'static,
//         F::Output: Send + 'static,
//     {
//         self.tasks.spawn(task)
//     }

//     #[track_caller]
//     #[inline]
//     pub fn spawn_blocking<F, T>(&self, task: F) -> JoinHandle<T>
//     where
//         F: FnOnce() -> T,
//         F: Send + 'static,
//         T: Send + 'static,
//     {
//         self.tasks.spawn_blocking(task)
//     }

//     #[must_use]
//     #[inline]
//     pub async fn spawn_blocking_result<F, T>(&self, task: F) -> Result<T>
//     where
//         F: FnOnce() -> Result<T>,
//         F: Send + 'static,
//         T: Send + 'static,
//     {
//         use sentry::Hub;
//         use std::convert::identity;

//         let current_span = tracing::Span::current();
//         let hub = sentry::Hub::current();
//         self.tasks
//             .spawn_blocking(move || current_span.in_scope(|| Hub::run(hub, task)))
//             .await
//             .map_err(|e: tokio::task::JoinError| anyhow!(e))
//             .and_then(identity)
//     }

//     #[must_use]
//     pub fn remaining_tasks(&self) -> usize {
//         self.tasks.len()
//     }

//     pub async fn close_tasks_and_wait(&self) {
//         let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
//         self.tasks.close();
//         loop {
//             match futures::future::select(Box::pin(interval.tick()), Box::pin(self.tasks.wait()))
//                 .await
//             {
//                 Either::Left((..)) => {
//                     tracing::info!("Waiting for {} task/s to be completed", self.tasks.len());
//                 }
//                 Either::Right(..) => {
//                     tracing::info!("All tasks are completed");
//                     break;
//                 }
//             }
//         }
//     }
// }

// impl App {
//     #[must_use]
//     pub fn has_shutdown(&self) -> bool {
//         self.shutdown.is_cancelled()
//     }

//     pub fn perform_shutdown(&self, reason: AppShutdownReason) {
//         if self.has_shutdown() {
//             return;
//         }

//         if matches!(reason, AppShutdownReason::ShutdownSignal) {
//             tracing::info!("{reason}; performing graceful shutdown...");
//         } else {
//             tracing::warn!("{reason}; performing graceful shutdown...");
//         }

//         self.shutdown.cancel();
//     }

//     pub fn shutdown_signal(&self) -> WaitForCancellationFuture<'_> {
//         self.shutdown.cancelled()
//     }
// }
