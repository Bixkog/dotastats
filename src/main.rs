mod data_retriever;
mod input_cache;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_retriever = data_retriever::DataRetriever::new();
    data_retriever.get_guild_players_matches_info(&"316887".to_string()).await?;
    Ok(())
}