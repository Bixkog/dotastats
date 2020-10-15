#![feature(try_trait)]
mod analyzers;
mod constants;
mod data_retriever;
mod extractor;
mod match_stats;
mod match_storage;
mod match_updater;
mod opendota_client;
mod parse_requester;
mod types;
mod utils;
use clap::Clap;

#[derive(Clap)]
#[clap(version = "0.1", author = "woziebly")]
struct Opts {
    #[clap(short, long, default_value = "316887")]
    guild_id: String,
    #[clap(short, long)]
    update: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();
    if opts.update {
        let match_updater = match_updater::MatchUpdater::new();
        match_updater.try_update(&opts.guild_id).await?;
    }
    let data_retriever = data_retriever::DataRetriever::new();
    let guild_raw_data = data_retriever.get_guild_raw_data(&opts.guild_id).await?;
    let parse_requester = parse_requester::ParseRequester::new();
    parse_requester
        .request_parsing(&guild_raw_data.guild_id, &guild_raw_data.members_matches)
        .await?;
    let matches = extractor::extract_stats(guild_raw_data).unwrap();
    let team_setup_rank = analyzers::team::best_teams(&matches);
    for (team_setup, winratio) in team_setup_rank.iter() {
        if winratio.wins + winratio.looses < 30 {
            continue;
        }
        println!(
            "Team: {} wins: {} looses: {} winratio: {:.3}",
            team_setup,
            winratio.wins,
            winratio.looses,
            winratio.as_percent()
        );
    }
    let synergies = analyzers::synergy::lane_synergy(&matches);
    for (lane, (synergy, [wr, p1_wr, p2_wr])) in synergies {
        println!(
            "Lane: {} Synergy: {} Duo_wr: {:?} Solo_wrs: {:?}, {:?}",
            lane, synergy, wr, p1_wr, p2_wr
        );
    }
    let roles_score = analyzers::roles::best_roles(&matches);
    for (roles, score) in roles_score.iter().take(100) {
        println!("{} -> {:?}", roles, score)
    }
    Ok(())
}
