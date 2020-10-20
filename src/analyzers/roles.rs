use super::analyzers_utils::*;
use crate::heroes_info::{Hero, HeroesInfo};
use crate::match_stats::Match;
use crate::match_stats::PlayerName;
use std::collections::HashMap;
use crate::CONFIG;

pub type Roles = Vec<(PlayerName, String)>;
pub type RolesWr = Vec<(Roles, WinRatio)>;

/// Finds heroes played by players.
fn get_heroes(
    heroes_info: &HeroesInfo,
    match_: &Match,
    mut team: Vec<String>,
) -> Vec<(PlayerName, Hero)> {
    let mut team_setup = vec![];
    team.sort();
    for player in team {
        let player_hero_id = skip_fail!(match_.get_player_hero(&player));
        let hero = heroes_info.get_hero(player_hero_id);
        team_setup.push((player, hero));
    }
    team_setup
}

/// As every hero can have several roles, this function generates subsets of those roles for a team.
/// e.g. P1: [A, B], P2: [B, C] -> ["P1-A P2-B", "P1-A P2-C", "P1-B P2-B", "P1-B P2-C"]
fn get_role_subsets(team_setup: Vec<(PlayerName, Hero)>) -> Vec<Roles> {
    let mut role_subsets = vec![Vec::new()];
    for (player, hero) in team_setup.iter() {
        role_subsets = [
            role_subsets.clone(),
            role_subsets
                .into_iter()
                .flat_map(|roles| {
                    hero.roles.iter().map(move |role| {
                        let mut roles_new = roles.clone();
                        roles_new.push((player.clone(), role.clone()));
                        roles_new
                    })
                })
                .collect(),
        ]
        .concat()
    }
    role_subsets
}

pub fn get_roles_wr(matches: &Vec<Match>) -> RolesWr {
    let heroes_info_filename = CONFIG.get_str("heroes_info_filename").unwrap().to_string();
    let heroes_info = HeroesInfo::init(heroes_info_filename);
    let mut roles_score: HashMap<Roles, WinRatio> = HashMap::new();
    for match_ in matches {
        let team = skip_fail!(match_.get_team());
        let team_setup = get_heroes(&heroes_info, match_, team);
        let role_subsets = get_role_subsets(team_setup);
        let is_won = skip_fail!(match_.is_won());
        for subset in role_subsets {
            if subset.is_empty() {
                continue;
            }
            roles_score.entry(subset).or_default().add_score(is_won);
        }
    }
    let mut result: RolesWr = roles_score.into_iter().collect();
    result = result.into_iter().collect();
    result.sort_by_key(|(_, wr)| wr.clone());
    result.reverse();
    result
}

pub fn roles_synergies(matches: &Vec<Match>) -> Vec<(Roles, (f64, u32))> {
    let roles_wr_res: RolesWr = get_roles_wr(matches)
        .into_iter()
        .filter(|(_, wr)| wr.total() > 30)
        .collect();
    let single_wr = roles_wr_res.iter().filter(|(r, _)| r.len() == 1).fold(
        HashMap::<(String, String), WinRatio>::new(),
        |mut s, (roles, wr)| {
            s.insert(roles[0].clone(), wr.clone());
            s
        },
    );
    let mut result = vec![];
    for (roles, team_wr) in roles_wr_res {
        if roles.len() <= 1 {
            continue;
        }
        let mut avg_solo_wr = 0.;
        for role in roles.iter() {
            avg_solo_wr += single_wr[role].as_percent();
        }
        if avg_solo_wr == 0. {
            continue;
        }
        avg_solo_wr = avg_solo_wr / roles.len() as f64;
        result.push((roles, (team_wr.as_percent() / avg_solo_wr, team_wr.total())));
    }
    result.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
    result
}

pub fn compress_roles_wr(roles_wr: RolesWr) -> RolesWr {
    let relevant_total_games = CONFIG.get_int("min_roles_wr_games").unwrap() as u32;
    roles_wr
        .into_iter()
        .filter(|(_, wr)| wr.total() >= relevant_total_games)
        .collect()
}

pub fn roles_wr_to_json(result: RolesWr) -> serde_json::Value {
    serde_json::to_value(result).unwrap()
}
