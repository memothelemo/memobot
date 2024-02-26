use twilight_gateway::ShardId;

use crate::app::App;

#[derive(Debug, Clone)]
pub struct Context {
    app: App,
    shard_id: ShardId,
}

impl Context {
    #[must_use]
    pub fn new(app: &App, shard_id: ShardId) -> Self {
        Self {
            app: app.clone(),
            shard_id,
        }
    }

    #[must_use]
    pub fn app(&self) -> &App {
        &self.app
    }

    #[must_use]
    pub fn shard_id(&self) -> ShardId {
        self.shard_id
    }
}
