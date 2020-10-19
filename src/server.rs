use rocket;
use crate::request_processing;
use crate::storage::result_storage;
use rocket::response::content;

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
async fn process_guild(guild_id: String) -> Option<()> {
    match request_processing::process_guild_request(&guild_id, /*update:*/ false).await {
        Ok(()) => Some(()),
        Err(e) => {
            println!("Error during processing guild data: {}", e);
            return None
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
    rocket::ignite().mount("/dotastats", routes![roles_wr_req, process_guild, start, stop, health]).launch().await?;
    Ok(())
}