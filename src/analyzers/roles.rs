use super::analyzers_utils::*;
use crate::constants::{Constants, Hero};
use crate::match_stats::Match;
use crate::match_stats::PlayerName;
use std::collections::HashMap;

/// Finds heroes played by players.
fn get_heroes(
    constants: &Constants,
    match_: &Match,
    mut team: Vec<String>,
) -> Vec<(PlayerName, Hero)> {
    let mut team_setup = vec![];
    team.sort();
    for player in team {
        let player_hero_id = skip_fail!(match_.get_player_hero(&player));
        let hero = constants.get_hero(player_hero_id);
        team_setup.push((player, hero));
    }
    team_setup
}

/// As every hero can have several roles, this function generates subsets of those roles for a team.
/// e.g. P1: [A, B], P2: [B, C] -> ["P1-A P2-B", "P1-A P2-C", "P1-B P2-B", "P1-B P2-C"]
fn get_role_subsets(team_setup: Vec<(PlayerName, Hero)>) -> Vec<String> {
    let mut role_subsets = vec![String::new()];
    for (player, hero) in team_setup.iter() {
        role_subsets = [role_subsets.clone(), role_subsets
            .into_iter()
            .flat_map(|s| {
                hero.roles
                    .iter()
                    .map(move |role| format!("{} {}-{}", s.clone(), player, role))
            })
            .collect()].concat()
    }
    role_subsets
}

pub fn best_roles(data: &Vec<Match>) -> Vec<(String, WinRatio)> {
    // TODO: init earlier and pass
    let constants = Constants::init("heroes.txt".to_string());
    let mut roles_score: HashMap<String, WinRatio> = HashMap::new();
    for match_ in data {
        let team = skip_fail!(match_.get_team());
        let team_setup = get_heroes(&constants, match_, team);
        let role_subsets = get_role_subsets(team_setup);
        let is_won = skip_fail!(match_.is_won());
        for subset in role_subsets {
            if subset == "" {continue}
            roles_score.entry(subset).or_default().add_score(is_won);
        }
    }
    let mut result: Vec<(String, WinRatio)> = roles_score.into_iter().collect();
    result = result
        .into_iter()
        .filter(|(_, wr)| wr.total() > 30)
        .collect();
    result.sort_by_key(|(_, wr)| wr.clone());
    result.reverse();
    result
}
