#![feature(try_trait)]
mod analyzers;
mod constants;
mod data_retrieval;
mod match_stats;
mod storage;
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
    let matches = data_retrieval::retrieval_agent::process_guild_matches_retrieval(opts.guild_id, opts.update).await?;
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
    let mut roles_score = analyzers::roles::roles_wr(&matches);
    println!("BEST ROLES");
    for (roles, score) in roles_score.iter().take(100) {
        if score.total() < 30 {continue}
        println!("{:?} -> {:?}", roles, score)
    }
    roles_score.reverse();
    println!("WORST ROLES");
    for (roles, score) in roles_score.iter().take(100) {
        println!("{:?} -> {:?}", roles, score)
    }
    analyzers::roles::plot(&roles_score).unwrap();
    let mut roles_synergies = analyzers::roles::roles_synergies(&matches);
    println!("BEST ROLES SYNERGIES");
    for (roles, (synergy, games)) in roles_synergies.iter().take(100) {
        println!("{:?} -> Synergy: {} Games: {} ", roles, synergy, games);
    }
    roles_synergies.reverse();
    println!("WORST ROLES SYNERGIES");
    for (roles, (synergy, games)) in roles_synergies.iter().take(100) {
        println!("{:?} -> Synergy: {} Games: {} ", roles, synergy, games);
    }
    Ok(())
}
