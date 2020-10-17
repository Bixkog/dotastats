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

pub async fn start() -> Result<(), Box<dyn std::error::Error>>  {
    rocket::ignite().mount("/dotastats", routes![guild]).launch().await?;
    Ok(())
}