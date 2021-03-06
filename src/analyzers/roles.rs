use crate::analyzers::analyzers_utils::*;
use crate::analyzers::WinRatio;
use crate::heroes_info::{Hero, HeroesInfo};
use crate::match_stats::Match;
use crate::match_stats::PlayerName;
use crate::CONFIG;
use serde::Serialize;
use std::collections::HashMap;

pub type RoleName = String;
pub type Roles = Vec<(PlayerName, RoleName)>;
pub type RolesWr = Vec<(Roles, WinRatio)>;
pub type RolesSynergyResult = f64;

/// As every hero can have several roles, this function generates subsets of those roles for a team.
/// e.g. P1: [A, B], P2: [B, C] -> [(P1-A, P2-B), (P1-A, P2-C), (P1-B, P2-B), (P1-B, P2-C)].
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

/// Computes winratio for each Player-Role setup.
pub fn get_roles_wr(matches: &Vec<Match>) -> RolesWr {
    let heroes_info_filename = CONFIG.get_str("heroes_info_filename").unwrap().to_string();
    let heroes_info = match HeroesInfo::init(heroes_info_filename) {
        Ok(val) => val,
        Err(e) => {
            warn!("Can't load heroes_info from heroes.json: {}", e);
            return vec![];
        }
    };
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
    roles_score.into_iter().collect()
}

/// Computes synergy factor for each Player-Role setup. Synergy is equal to setup winratio
/// divided by average of each player in the setup (player-role) individual winratio.
pub fn get_roles_synergies(roles_wr: &RolesWr) -> Vec<(Roles, RolesSynergyResult)> {
    let single_wr = roles_wr.iter().filter(|(r, _)| r.len() == 1).fold(
        HashMap::<(String, String), WinRatio>::new(),
        |mut s, (roles, wr)| {
            s.insert(roles[0].clone(), wr.clone());
            s
        },
    );
    let relevant_total_games = CONFIG.get_int("min_roles_wr_games").unwrap() as u32;
    roles_wr
        .iter()
        .filter_map(|(roles, team_wr)| {
            if roles.len() <= 1 || team_wr.total() < relevant_total_games {
                return None;
            }
            let mut avg_solo_wr = 0.;
            for role in roles.iter() {
                avg_solo_wr += (single_wr[role].clone() - team_wr.clone())
                    .expect("Single wr does no contain some team wr.")
                    .as_percent();
            }
            avg_solo_wr = avg_solo_wr / roles.len() as f64;
            let synergy = team_wr.as_percent() / avg_solo_wr;
            let synergy = (synergy * 1000.).round() / 1000.;
            Some((roles.clone(), synergy))
        })
        .collect()
}

/// Struct containing results for certain records of Player-Role setups.
/// best_single - player with highest winratio for certain role.
/// top3_carry_sup - top3 of setups (P1-Carry, P2-Support).
/// best_fight_crew - top 3 player setup containing roles such as Disabler, Initiator, Nuker,
///                     Support, Durable.
/// best_nuking_squad - top 2 Nuker setup.
#[derive(Serialize)]
pub struct RolesRecords {
    pub best_single: Vec<(PlayerName, RoleName, WinRatio)>,
    pub top3_carry_sup: [(PlayerName, PlayerName, WinRatio); 3],
    pub best_fight_crew: (PlayerName, PlayerName, PlayerName, WinRatio),
    pub best_nuking_squad: (PlayerName, PlayerName, WinRatio),
}

impl RolesRecords {
    /// Constructs RolesRecords using Player-Role setups winratio.
    pub fn extract_records(roles_wr: &RolesWr) -> Self {
        let relevant_total_games = CONFIG.get_int("min_roles_wr_games").unwrap() as u32;
        let roles_wr = roles_wr
            .iter()
            .filter(|(_, wr)| wr.total() > relevant_total_games)
            .map(|t| t.clone())
            .collect();
        let best_single = RolesRecords::extract_best_single(&roles_wr);
        let top3_carry_sup = RolesRecords::extract_top3_carry_sup(&roles_wr);
        let best_fight_crew = RolesRecords::extract_best_fight_crew(&roles_wr);
        let best_nuking_squad = RolesRecords::extract_best_nuking_squad(&roles_wr);
        RolesRecords {
            best_single,
            top3_carry_sup,
            best_fight_crew,
            best_nuking_squad,
        }
    }

    /// Calculates best_single field.
    fn extract_best_single(roles_wr: &RolesWr) -> Vec<(PlayerName, RoleName, WinRatio)> {
        roles_wr
            .iter()
            .filter(|(roles, _)| roles.len() == 1)
            .fold(
                HashMap::<&RoleName, Vec<(&PlayerName, &WinRatio)>>::new(),
                |mut s, (roles, wr)| {
                    s.entry(&roles[0].1)
                        .or_insert(vec![])
                        .push((&roles[0].0, wr));
                    s
                },
            )
            .into_iter()
            .map(|(role, players_wr)| {
                let (player, wr) = match players_wr.into_iter().max_by_key(|p| p.1) {
                    Some((player_max, wr_max)) => (player_max.clone(), wr_max.clone()),
                    None => Default::default(),
                };
                (role.clone(), player.clone(), wr.clone())
            })
            .collect()
    }

    /// Calculates top3_carry_sup field.
    fn extract_top3_carry_sup(roles_wr: &RolesWr) -> [(PlayerName, PlayerName, WinRatio); 3] {
        let mut carry_sup: Vec<(PlayerName, PlayerName, WinRatio)> = roles_wr
            .iter()
            .filter(|(roles, _)| roles.len() == 2)
            .filter(|(roles, _)| {
                let mut r: Vec<&str> = roles.iter().map(|p| p.1.as_str()).collect();
                r.sort();
                r == vec!["Carry", "Support"]
            })
            .map(|(roles, wr)| (roles[0].0.clone(), roles[1].0.clone(), wr.clone()))
            .collect();
        carry_sup.sort_by_key(|t| t.2.clone());
        carry_sup.reverse();
        let mut res: [(PlayerName, PlayerName, WinRatio); 3] = Default::default();
        for i in 0..(std::cmp::min(3, carry_sup.len())) {
            res[i] = carry_sup[i].clone();
        }
        res
    }

    /// Calculates best_fight_crew field.
    fn extract_best_fight_crew(
        roles_wr: &RolesWr,
    ) -> (PlayerName, PlayerName, PlayerName, WinRatio) {
        let best_fight_crew = roles_wr
            .iter()
            .filter(|(roles, _)| roles.len() == 3)
            .filter(|(roles, _)| {
                let mut r: Vec<&str> = roles.iter().map(|p| p.1.as_str()).collect();
                r.sort();
                [
                    vec!["Disabler", "Initiator", "Nuker"],
                    vec!["Initiator", "Nuker", "Support"],
                    vec!["Disabler", "Initiator", "Support"],
                    vec!["Disabler", "Durable", "Support"],
                    vec!["Durable", "Initiator", "Support"],
                ]
                .contains(&r)
            })
            .max_by_key(|(_, wr)| wr);
        match best_fight_crew {
            Some((roles, wr)) => (
                roles[0].0.clone(),
                roles[1].0.clone(),
                roles[2].0.clone(),
                wr.clone(),
            ),
            None => Default::default(),
        }
    }

    /// Calculates best_nuking field.
    fn extract_best_nuking_squad(roles_wr: &RolesWr) -> (PlayerName, PlayerName, WinRatio) {
        let squad = roles_wr
            .iter()
            .filter(|(roles, _)| roles.len() == 2)
            .filter(|(roles, _)| {
                let r: Vec<&str> = roles.iter().map(|p| p.1.as_str()).collect();
                r == vec!["Nuker", "Nuker"]
            })
            .max_by_key(|(_, wr)| wr);
        match squad {
            Some((roles, wr)) => (roles[0].0.clone(), roles[1].0.clone(), wr.clone()),
            None => Default::default(),
        }
    }
}

pub fn get_roles_records(roles_wr: &RolesWr) -> RolesRecords {
    RolesRecords::extract_records(roles_wr)
}

/// Drops setups with not enough total games. Used after calculating other stats.
pub fn compress_roles_wr(roles_wr: RolesWr) -> RolesWr {
    let relevant_total_games = CONFIG.get_int("min_roles_wr_games").unwrap() as u32;
    roles_wr
        .into_iter()
        .filter(|(_, wr)| wr.total() >= relevant_total_games)
        .collect()
}
