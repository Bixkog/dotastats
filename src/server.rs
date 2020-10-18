use rocket;
use crate::request_processing;
use crate::storage::result_storage;

#[get("/guild/<guild_id>")]
async fn guild(guild_id: String) -> Option<String> {
    match result_storage::get_roles_wr_results(&guild_id) {
        Ok(payload) => return Some(payload),
        Err(e) => {
            println!("Error during the reading of roles_wr result: {}. Recomputing roles_wr.", e);
        }
    };
    match request_processing::process_guild_request(&guild_id, /*update:*/ false).await {
        Ok(()) => (),
        Err(e) => {
            println!("Error during processing guild data: {}", e);
            return None
        }
    };
    match result_storage::get_roles_wr_results(&guild_id) {
        Ok(payload) => Some(payload),
        Err(e) => {
            println!("Error during the reading of roles_wr results: {}.", e);
            None
        }
    }
}

#[get("/start")]
async fn start() -> &'static str {
    "OK!"
}

#[get("/stop")]
async fn stop() -> &'static str {
    "OK!"
}

#[get("/health")]
async fn health() -> &'static str {
    "OK!"
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>>  {
    rocket::ignite().mount("/dotastats", routes![guild, start, stop, health]).launch().await?;
    Ok(())
}