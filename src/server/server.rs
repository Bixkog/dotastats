use crate::server::data_processing::{self, DPQ};
use crate::server::data_updater;
use crate::server::health_routes::{health, start, stop};
use crate::storage::{result_storage::AnalysisTag, Storage};
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
        Ok(Some(payload)) => Some(content::Json(payload)),
        Ok(None) => {
            println!("roles_wr result not found");
            None
        }
        Err(e) => {
            println!("Error during the reading of roles_wr result: {}", e);
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
        Ok(Some(payload)) => Some(content::Json(payload)),
        Ok(None) => {
            println!("roles_synergy result not found");
            None
        }
        Err(e) => {
            println!("Error during the reading of roles_synergy result: {}", e);
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
        Ok(Some(payload)) => Some(content::Json(payload)),
        Ok(None) => {
            println!("roles_records result not found");
            None
        }
        Err(e) => {
            println!("Error during the reading of roles_records result: {}", e);
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
        Ok(Some(payload)) => Some(content::Json(payload)),
        Ok(None) => {
            println!("heroes_players_stats result not found");
            None
        }
        Err(e) => {
            println!(
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
        Ok(Some(payload)) => Some(content::Json(payload)),
        Ok(None) => {
            println!("players_wr result not found");
            None
        }
        Err(e) => {
            println!("Error during the reading of players_wr result: {}", e);
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
    match storage.is_guild_stats_ready(&guild_id).await {
        Ok(true) => return (),
        Ok(false) => (),
        Err(e) => {
            println!(
                "Error during checking if guild is processed: {}. Recomputing.",
                e
            );
            ()
        }
    }
    if data_processing_queue.read().await.contains(&guild_id) {
        println!("Guild {} is already in queue.", guild_id);
    } else {
        println!("Guild {} added to queue.", guild_id);
        data_processing_queue.write().await.push_back(guild_id);
    }
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
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
