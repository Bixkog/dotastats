use crate::analyzers::heroes::{get_hero_players_stats, get_heroes_played};
use crate::analyzers::players::get_players_wr;
use crate::analyzers::roles::{
    compress_roles_wr, get_roles_records, get_roles_synergies, get_roles_wr,
};
use crate::data_retrieval::retrieval_agent::process_guild_matches_retrieval;
use crate::match_stats::Match;
use crate::storage::result_storage::AnalysisTag;
use crate::storage::Storage;
use crate::BoxError;
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

/// data processing queue
pub type DPQ = Arc<RwLock<VecDeque<String>>>;

async fn process_roles_wr(
    storage: Arc<Storage>,
    guild_id: &String,
    matches: &Vec<Match>,
) -> Result<(), BoxError> {
    let roles_wr = get_roles_wr(&matches);
    let roles_synergy = get_roles_synergies(&roles_wr);
    let roles_records = get_roles_records(&roles_wr);

    let roles_wr = compress_roles_wr(roles_wr);
    let roles_wr_json = serde_json::to_value(roles_wr)?;
    let roles_synergy_json = serde_json::to_value(roles_synergy)?;
    let roles_records_json = serde_json::to_value(roles_records)?;
    storage
        .store_result(guild_id, roles_wr_json, AnalysisTag::RolesWr)
        .await?;
    storage
        .store_result(guild_id, roles_synergy_json, AnalysisTag::RolesSynergy)
        .await?;
    storage
        .store_result(guild_id, roles_records_json, AnalysisTag::RolesRecords)
        .await?;
    Ok(())
}

async fn process_heroes_data(
    storage: Arc<Storage>,
    guild_id: &String,
    matches: &Vec<Match>,
) -> Result<(), BoxError> {
    let heroes_played = get_heroes_played(&matches);
    let heroes_players_stats = get_hero_players_stats(&heroes_played);
    let heroes_players_stats_json = serde_json::to_value(heroes_players_stats)?;
    storage
        .store_result(
            guild_id,
            heroes_players_stats_json,
            AnalysisTag::HeroesPlayersStats,
        )
        .await?;
    Ok(())
}

async fn process_players_data(
    storage: Arc<Storage>,
    guild_id: &String,
    matches: &Vec<Match>,
) -> Result<(), BoxError> {
    let players_wr = get_players_wr(matches);
    let players_wr_json = serde_json::to_value(players_wr)?;
    storage
        .store_result(guild_id, players_wr_json, AnalysisTag::PlayersWr)
        .await?;
    Ok(())
}

async fn process_guild_data(storage: Arc<Storage>, guild_id: &String) -> Result<(), BoxError> {
    let matches = process_guild_matches_retrieval(storage.clone(), guild_id).await?;
    process_roles_wr(storage.clone(), &guild_id, &matches).await?;
    process_heroes_data(storage.clone(), &guild_id, &matches).await?;
    process_players_data(storage.clone(), &guild_id, &matches).await?;
    Ok(())
}

pub async fn spawn_worker(queue: DPQ, storage: Arc<Storage>) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            while queue.read().await.len() > 0 {
                let guild_id = match queue.read().await.front() {
                    Some(val) => val.clone(),
                    None => {
                        error!("Unable to aquire front from processing queue.");
                        break;
                    }
                };
                match process_guild_data(storage.clone(), &guild_id).await {
                    Ok(()) => info!("Finished processing guild: {}", &guild_id),
                    Err(e) => warn!("Error during processing guild data: {}", e),
                };
                queue.write().await.pop_front();
            }
            tokio::time::delay_for(tokio::time::Duration::from_secs(1)).await;
        }
    })
}
