use memobot_kernel::Kernel;
use twilight_gateway::ShardId;

#[derive(Debug, Clone)]
pub struct Context {
    kernel: Kernel,
    shard_id: ShardId,
}

impl Context {
    #[must_use]
    pub fn new(kernel: &Kernel, shard_id: ShardId) -> Self {
        Self {
            kernel: kernel.clone(),
            shard_id,
        }
    }

    #[must_use]
    pub fn kernel(&self) -> &Kernel {
        &self.kernel
    }

    #[must_use]
    pub fn shard_id(&self) -> ShardId {
        self.shard_id
    }
}
