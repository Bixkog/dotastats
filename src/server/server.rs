use crate::server::data_updater;
use crate::server::health_routes::{health, start, stop};
use crate::storage::{
    result_storage::{AnalysisTag, GuildResultsState, ResultsState},
    Storage,
};
use crate::BoxError;
use crate::{
    server::data_processing::{self, DPQ},
    storage::result_storage,
};
use rocket;
use rocket::response::content;
use rocket::State;
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::RwLock;

#[get("/guild/roles_wr/<guild_id>")]
async fn roles_wr_req<'a>(
    guild_id: String,
    storage: State<'a, Storage>,
) -> Option<content::Json<String>> {
    match storage.get_result(&guild_id, AnalysisTag::RolesWr).await {
        Ok(payload) => Some(content::Json(payload)),
        Err(e) => {
            warn!("Error during the reading of roles_synergy result: {}", e);
            None
        }
    }
}

#[get("/guild/roles_synergy/<guild_id>")]
async fn roles_synergy_req<'a>(
    guild_id: String,
    storage: State<'a, Storage>,
) -> Option<content::Json<String>> {
    match storage
        .get_result(&guild_id, AnalysisTag::RolesSynergy)
        .await
    {
        Ok(payload) => Some(content::Json(payload)),
        Err(e) => {
            warn!("Error during the reading of roles_synergy result: {}", e);
            None
        }
    }
}

#[get("/guild/roles_records/<guild_id>")]
async fn roles_records_req<'a>(
    guild_id: String,
    storage: State<'a, Storage>,
) -> Option<content::Json<String>> {
    match storage
        .get_result(&guild_id, AnalysisTag::RolesRecords)
        .await
    {
        Ok(payload) => Some(content::Json(payload)),
        Err(e) => {
            warn!("Error during the reading of roles_records result: {}", e);
            None
        }
    }
}

#[get("/guild/heroes_players_stats/<guild_id>")]
async fn heroes_players_stats_req<'a>(
    guild_id: String,
    storage: State<'a, Storage>,
) -> Option<content::Json<String>> {
    match storage
        .get_result(&guild_id, AnalysisTag::HeroesPlayersStats)
        .await
    {
        Ok(payload) => Some(content::Json(payload)),
        Err(e) => {
            warn!(
                "Error during the reading of heroes_players_stats result: {}",
                e
            );
            None
        }
    }
}

#[get("/guild/players_wr/<guild_id>")]
async fn players_wr_req<'a>(
    guild_id: String,
    storage: State<'a, Storage>,
) -> Option<content::Json<String>> {
    match storage.get_result(&guild_id, AnalysisTag::PlayersWr).await {
        Ok(payload) => Some(content::Json(payload)),
        Err(e) => {
            warn!("Error during the reading of players_wr result: {}", e);
            None
        }
    }
}

#[post("/guild/process/<guild_id>")]
async fn process_guild<'a>(
    guild_id: String,
    data_processing_queue: State<'a, DPQ>,
    storage: State<'a, Storage>,
) -> () {
    match storage.get_guild_results_state(&guild_id).await {
        Ok(GuildResultsState {
            guild_id: storage_guild_id,
            state,
        }) => {
            if guild_id != storage_guild_id {
                error!("Unable to receive correct guild results state from storage.");
                return ();
            }
            match state {
                ResultsState::NotComputed => {}
                ResultsState::ResultMissing => {}
                ResultsState::AllComputed { timestamp: _ } => return (),
            }
        }
        Err(e) => {
            warn!(
                "Couldn't find results timestamp: {}. Recomputing results.",
                e
            );
            ()
        }
    }
    if data_processing_queue.read().await.contains(&guild_id) {
        info!("Guild {} is already in queue.", guild_id);
    } else {
        info!("Guild {} added to queue.", guild_id);
        data_processing_queue.write().await.push_back(guild_id);
    }
}

pub async fn run() -> Result<(), BoxError> {
    let data_processing_queue: DPQ = Arc::new(RwLock::new(VecDeque::new()));
    let storage = Storage::from_config().await?;
    let data_processor = data_processing::spawn_worker(data_processing_queue.clone());
    let updater = data_updater::spawn_worker(data_processing_queue.clone());
    updater.await;
    data_processor.await;
    rocket::ignite()
        .mount(
            "/dotastats",
            routes![
                roles_wr_req,
                roles_synergy_req,
                roles_records_req,
                heroes_players_stats_req,
                players_wr_req,
                process_guild,
                start,
                stop,
                health
            ],
        )
        .manage(data_processing_queue)
        .manage(storage)
        .launch()
        .await?;
    Ok(())
}
