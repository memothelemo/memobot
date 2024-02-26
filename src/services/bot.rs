use tokio::task::JoinSet;

use crate::app::App;

pub async fn start(app: App, shards: Vec<twilight_gateway::Shard>) {
    tracing::info!("Starting bot with {} shard(s)", shards.len());

    let mut running_shards = JoinSet::new();
    let total_shards = shards.len();

    for shard in shards {
        running_shards.spawn(crate::bot::shard::main(app.clone(), shard));
    }

    app.shutdown_guard().await;

    tracing::info!("Closing bot service...");
    loop {
        let finished_shards = total_shards - running_shards.len();
        tracing::info!("Waiting for {finished_shards}/{total_shards} shards(s) to finish",);

        if running_shards.join_next().await.is_none() {
            break;
        }
    }
}
