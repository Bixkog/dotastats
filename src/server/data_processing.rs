use crate::analyzers::players::get_players_wr;
use crate::analyzers::roles::{
    compress_roles_wr, get_roles_records, get_roles_synergies, get_roles_wr,
};
use crate::data_retrieval::retrieval_agent::process_guild_matches_retrieval;
use crate::match_stats::Match;
use crate::storage::result_storage::{
    store_heroes_players_stats_result, store_players_wr_result, store_roles_records_result,
    store_roles_synergy_result, store_roles_wr_result,
};
use crate::{
    analyzers::heroes::{get_hero_players_stats, get_heroes_played},
    storage::Storage,
};
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

/// data processing queue
pub type DPQ = Arc<RwLock<VecDeque<String>>>;

fn process_roles_wr(
    guild_id: &String,
    matches: &Vec<Match>,
) -> Result<(), Box<dyn std::error::Error>> {
    let roles_wr = get_roles_wr(&matches);
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

fn process_heroes_data(
    guild_id: &String,
    matches: &Vec<Match>,
) -> Result<(), Box<dyn std::error::Error>> {
    let heroes_played = get_heroes_played(&matches);
    let heroes_players_stats = get_hero_players_stats(&heroes_played);
    let heroes_players_stats_json = serde_json::to_value(heroes_players_stats)?;
    store_heroes_players_stats_result(guild_id, heroes_players_stats_json)?;
    Ok(())
}

fn process_players_data(
    guild_id: &String,
    matches: &Vec<Match>,
) -> Result<(), Box<dyn std::error::Error>> {
    let players_wr = get_players_wr(matches);
    let players_wr_json = serde_json::to_value(players_wr)?;
    store_players_wr_result(guild_id, players_wr_json)?;
    Ok(())
}

async fn process_guild_data(
    storage: Arc<Storage>,
    guild_id: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let matches = process_guild_matches_retrieval(storage, guild_id).await?;
    process_roles_wr(&guild_id, &matches)?;
    process_heroes_data(&guild_id, &matches)?;
    process_players_data(&guild_id, &matches)?;
    Ok(())
}

pub async fn spawn_worker(queue: DPQ) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let storage = match Storage::new().await {
                Ok(s) => s,
                Err(e) => {
                    println!("Unable to connect to database: {}", e);
                    break;
                }
            };
            let storage = Arc::new(storage);
            while queue.read().await.len() > 0 {
                let guild_id = queue.read().await.front().unwrap().clone();
                match process_guild_data(storage.clone(), &guild_id).await {
                    Ok(()) => println!("Finished processing guild: {}", &guild_id),
                    Err(e) => println!("Error during processing guild data: {}", e),
                };
                queue.write().await.pop_front();
            }
            tokio::time::delay_for(tokio::time::Duration::from_secs(1)).await;
        }
    })
}
