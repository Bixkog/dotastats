use rocket;
use rocket::response::content;
use rocket::State;
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::RwLock;
use crate::server::data_processing::{self, DPQ};
use crate::storage::result_storage;
use crate::server::results_updater;
use crate::server::health_routes::{health, start, stop};

#[get("/guild/roles_wr/<guild_id>")]
async fn roles_wr_req(guild_id: String) -> Option<content::Json<String>> {
    match result_storage::get_roles_wr_results(&guild_id) {
        Ok(payload) => Some(content::Json(payload)),
        Err(e) => {
            println!("Error during the reading of roles_wr result: {}", e);
            None
        }
    }
}

#[post("/guild/process/<guild_id>")]
async fn process_guild<'a>(guild_id: String, data_processing_queue: State<'a, DPQ>) -> () {
    match result_storage::is_guild_result_ready(&guild_id) {
        Ok(true) => return (),
        Ok(false) => (),
        Err(e) => {
            println!("Error during checking if guild is processed: {}. Recomputing.", e);
            ()
        }
    }
    let worker_task = (guild_id.clone(), false);
    if data_processing_queue.read().await.contains(&worker_task) {
        println!("Guild {} is already in queue.", guild_id);
    } else {
        println!("Guild {} added to queue.", guild_id);
        data_processing_queue.write().await.push_back(worker_task);
    }
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>>  {
    let data_processing_queue: DPQ = Arc::new(RwLock::new(VecDeque::new()));
    let data_processor = data_processing::spawn_worker(data_processing_queue.clone());
    let updater = results_updater::spawn_worker();
    updater.await;
    data_processor.await;
    rocket::ignite()
        .mount("/dotastats", routes![roles_wr_req, process_guild, start, stop, health])
        .manage(data_processing_queue)
        .launch().await?;
    Ok(())
}