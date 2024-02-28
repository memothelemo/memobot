use memobot_kernel::Kernel;
use std::sync::Arc;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct Service {
    config: Arc<Config>,
    kernel: Kernel,
}

impl Service {
    #[must_use]
    pub fn new(config: Config, kernel: Kernel) -> Self {
        Self {
            config: Arc::new(config),
            kernel,
        }
    }
}

impl Service {
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    #[must_use]
    pub fn kernel(&self) -> &Kernel {
        &self.kernel
    }
}
