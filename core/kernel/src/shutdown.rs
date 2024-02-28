use tokio_util::sync::WaitForCancellationFuture;

use super::Kernel;
use crate::ShutdownReason;

impl Kernel {
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
