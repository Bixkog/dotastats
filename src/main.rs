mod data_retriever;
mod input_cache;
mod utils;
mod extractor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_retriever = data_retriever::DataRetriever::new();
    let guild_raw_data = data_retriever.get_guild_raw_data(&"316887".to_string()).await?;
    let matches = extractor::extract_stats(guild_raw_data).unwrap();
    println!("{:?}", matches[0]);
    Ok(())
}