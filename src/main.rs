#![feature(try_trait)]
mod data_retriever;
mod match_storage;
mod utils;
mod extractor;
mod match_stats;
mod analyser;
mod parse_requester;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_retriever = data_retriever::DataRetriever::new();
    let guild_raw_data = data_retriever.get_guild_raw_data(&"316887".to_string()).await?;
    let parse_requester = parse_requester::ParseRequester::new();
    parse_requester.request_parsing(&guild_raw_data.guild_id, &guild_raw_data.members_matches).await?;
    let matches = extractor::extract_stats(guild_raw_data).unwrap();
    let team_setup_rank = analyser::best_team_setup(&matches);
    for (team_setup, winratio) in team_setup_rank.iter() {
        if winratio.0 + winratio.1 < 30 {
            continue;
        }
        if winratio.1 == 0 {
            println!("Team: {} wins: {} looses: {}", team_setup, winratio.0, winratio.1);
            continue;
        }
        println!("Team: {} wins: {} looses: {} winratio: {:.3}", team_setup, winratio.0, winratio.1, winratio.0 as f64 / (winratio.0 + winratio.1) as f64 );
    }
    Ok(())
}