use crate::analyzers::roles::{compress_roles_wr, get_roles_wr, roles_wr_to_json};
use crate::data_retrieval::retrieval_agent::process_guild_matches_retrieval;
use crate::match_stats::Match;
use crate::storage::result_storage::store_roles_wr_result;
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
    let roles_wr = compress_roles_wr(roles_wr);
    let roles_wr_json = roles_wr_to_json(roles_wr);
    store_roles_wr_result(guild_id, roles_wr_json)?;
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
