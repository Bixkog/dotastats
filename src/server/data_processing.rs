use crate::analyzers::roles::{
    compress_roles_wr, get_roles_records, get_roles_synergies, get_roles_wr,
};
use crate::data_retrieval::retrieval_agent::process_guild_matches_retrieval;
use crate::match_stats::Match;
use crate::storage::result_storage::{
    store_roles_records_result, store_roles_synergy_result, store_roles_wr_result,
};
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

/// data processing queue
pub type DPQ = Arc<RwLock<VecDeque<(String, bool)>>>;

fn process_roles_wr(
    guild_id: &String,
    data: &Vec<Match>,
) -> Result<(), Box<dyn std::error::Error>> {
    let roles_wr = get_roles_wr(&data);
    let roles_synergy = get_roles_synergies(&roles_wr);
    let roles_records = get_roles_records(&roles_wr);

    let roles_wr = compress_roles_wr(roles_wr);
    let roles_wr_json = serde_json::to_value(roles_wr).unwrap();
    let roles_synergy_json = serde_json::to_value(roles_synergy).unwrap();
    let roles_records_json = serde_json::to_value(roles_records).unwrap();
    store_roles_wr_result(guild_id, roles_wr_json)?;
    store_roles_synergy_result(guild_id, roles_synergy_json)?;
    store_roles_records_result(guild_id, roles_records_json)?;
    Ok(())
}

async fn process_guild_data(
    guild_id: &String,
    update: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let matches = process_guild_matches_retrieval(guild_id, update).await?;
    process_roles_wr(&guild_id, &matches)?;
    Ok(())
}

pub async fn spawn_worker(queue: DPQ) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            while queue.read().await.len() > 0 {
                let (guild_id, update) = queue.read().await.front().unwrap().clone();
                match process_guild_data(&guild_id, update).await {
                    Ok(()) => println!("Finished processing guild: {}", &guild_id),
                    Err(e) => println!("Error during processing guild data: {}", e),
                };
                queue.write().await.pop_front();
            }
            tokio::time::delay_for(tokio::time::Duration::from_secs(1)).await;
        }
    })
}
