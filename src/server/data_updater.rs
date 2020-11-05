use std::sync::Arc;

use crate::server::data_processing;
use crate::storage::{result_storage::ResultsState, Storage};
use crate::types::GuildId;
use crate::BoxError;
use crate::CONFIG;
use chrono::Utc;
use tokio::task::JoinHandle;

async fn update_results(guild_id: GuildId, data_processing_queue: data_processing::DPQ) {
    info!("Adding guild {} to update queue.", guild_id);
    data_processing_queue.write().await.push_back(guild_id);
}

async fn check_if_update(
    storage: Arc<Storage>,
    data_processing_queue: data_processing::DPQ,
) -> Result<(), BoxError> {
    let update_days = CONFIG.get_int("update_every_n_days").unwrap() as i64;
    let timestamp_now = Utc::now().timestamp();
    for guild_results_state in storage.get_guilds_results_state().await? {
        match guild_results_state.state {
            ResultsState::NotComputed => warn!(
                "Returned processed guild is not actually processed! guild_id: {}",
                guild_results_state.guild_id
            ),
            ResultsState::ResultMissing => {
                update_results(guild_results_state.guild_id, data_processing_queue.clone()).await
            }
            ResultsState::AllComputed { timestamp } => {
                if timestamp_now - timestamp > update_days * /*day in secs:*/ 86400 {
                    update_results(guild_results_state.guild_id, data_processing_queue.clone())
                        .await;
                }
            }
        };
    }
    Ok(())
}

pub async fn spawn_worker(
    data_processing_queue: data_processing::DPQ,
    storage: Arc<Storage>,
) -> Result<JoinHandle<()>, BoxError> {
    Ok(tokio::spawn(async move {
        loop {
            match check_if_update(storage.clone(), data_processing_queue.clone()).await {
                Ok(()) => (),
                Err(e) => {
                    error!("Error ({}) occured during update. Retry in 1 hour.", e);
                    ()
                }
            };
            tokio::time::delay_for(tokio::time::Duration::from_secs(3600)).await;
        }
    }))
}
