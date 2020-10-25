use super::analyzers_utils::*;
use crate::heroes_info::{Hero, HeroesInfo};
use crate::match_stats::Match;
use crate::match_stats::PlayerName;
use crate::CONFIG;
use itertools::Itertools;
use serde::Serialize;
use std::collections::HashMap;
extern crate ordered_float;
use ordered_float::OrderedFloat;

pub type HeroName = String;
pub type PlayerHeroScores = Vec<(Vec<(PlayerName, HeroName)>, WinRatio)>;

pub fn get_heroes_played(data: &Vec<Match>) -> PlayerHeroScores {
    let heroes_info_filename = CONFIG.get_str("heroes_info_filename").unwrap();
    let heroes_info = HeroesInfo::init(heroes_info_filename);
    let mut heroes_played: HashMap<Vec<(PlayerName, HeroName)>, WinRatio> = HashMap::new();
    for match_ in data.iter() {
        let team = skip_fail!(match_.get_team());
        let team_setup = get_heroes(&heroes_info, match_, team);
        let mut team_setup: Vec<(PlayerName, HeroName)> = team_setup
            .into_iter()
            .map(|(p, hero)| (p, hero.name))
            .collect();
        let is_won = skip_fail!(match_.is_won());
        for i in 1..6 {
            for team_combination in team_setup.clone().into_iter().combinations(i) {
                heroes_played
                    .entry(team_combination)
                    .or_default()
                    .add_score(is_won);
            }
        }
    }
    heroes_played.into_iter().collect()
}

#[derive(Serialize)]
pub struct HeroPlayersStats {
    heroes_stats: Vec<HeroStats>,
    players_stats: Vec<(PlayerName, HeroName, WinRatio)>,
}

#[derive(Serialize)]
pub struct HeroStats {
    pub hero_name: HeroName,
    pub top_player: (PlayerName, WinRatio, f64),
    pub common_player_raw: (PlayerName, WinRatio),
    pub common_player_relative: (PlayerName, WinRatio, f64),
}

pub fn get_hero_players_stats(player_hero_scores: &PlayerHeroScores) -> HeroPlayersStats {
    let hero_players_wr: HashMap<HeroName, Vec<(PlayerName, WinRatio)>> = player_hero_scores
        .iter()
        .filter(|(heroes_played, _)| heroes_played.len() == 1)
        .fold(
            HashMap::<HeroName, Vec<(PlayerName, WinRatio)>>::new(),
            |mut s, (heroes_played, wr)| {
                let hero_name = heroes_played[0].1.clone();
                let player_name = heroes_played[0].0.clone();
                s.entry(hero_name)
                    .or_default()
                    .push((player_name, wr.clone()));
                s
            },
        );
    let player_total_games: HashMap<PlayerName, u32> = player_hero_scores
        .iter()
        .filter(|(heroes_played, _)| heroes_played.len() == 1)
        .fold(
            HashMap::<PlayerName, u32>::new(),
            |mut s, (heroes_played, wr)| {
                let player_name = heroes_played[0].0.clone();
                *s.entry(player_name).or_default() += wr.total();
                s
            },
        );
    let player_hero_wr: HashMap<PlayerName, Vec<(HeroName, WinRatio)>> = player_hero_scores
        .iter()
        .filter(|(heroes_played, _)| heroes_played.len() == 1)
        .fold(
            HashMap::<PlayerName, Vec<(HeroName, WinRatio)>>::new(),
            |mut s, (heroes_played, wr)| {
                let hero_name = heroes_played[0].1.clone();
                let player_name = heroes_played[0].0.clone();
                s.entry(player_name)
                    .or_default()
                    .push((hero_name, wr.clone()));
                s
            },
        );
    let player_wr: HashMap<PlayerName, WinRatio> = player_hero_wr
        .iter()
        .map(|(player_name, hero_wr)| {
            (
                player_name.clone(),
                hero_wr
                    .iter()
                    .fold(WinRatio::default(), |wr, (_, hero_wr)| wr + hero_wr.clone()),
            )
        })
        .collect();
    let heroes_stats = hero_players_wr
        .into_iter()
        .filter_map(|(hero_name, players_wr)| {
            if players_wr.is_empty() {
                return None;
            };
            let top_player: (PlayerName, WinRatio, f64) = {
                let (player_name, wr) = match players_wr
                    .iter()
                    .filter(|(_, wr)| wr.total() > 5)
                    .max_by_key(|(player_name, wr)| {
                        OrderedFloat(wr.as_percent() / player_wr[player_name].as_percent())
                    }) {
                    Some(x) => x.clone(),
                    None => (String::new(), WinRatio::default()),
                };
                if player_name.is_empty() {
                    (String::new(), WinRatio::default(), 0.)
                } else {
                    let hero_relative_winratio =
                        wr.as_percent() / player_wr[&player_name].as_percent();
                    let hero_relative_winratio = (hero_relative_winratio * 1000.).round() / 1000.;
                    (player_name, wr, hero_relative_winratio)
                }
            };
            let common_player_raw: (PlayerName, WinRatio) = match players_wr
                .iter()
                .filter(|(_, wr)| wr.total() > 5)
                .max_by_key(|(_, wr)| wr.total())
            {
                Some(x) => x.clone(),
                None => (String::new(), WinRatio::default()),
            };
            let common_player_relative = {
                let (player_name, wr) = match players_wr
                    .iter()
                    .filter(|(_, wr)| wr.total() > 5)
                    .max_by_key(|(player_name, wr)| {
                        OrderedFloat(wr.total() as f64 / player_total_games[player_name] as f64)
                    }) {
                    Some(x) => x.clone(),
                    None => (String::new(), WinRatio::default()),
                };
                if player_name.is_empty() {
                    (String::new(), WinRatio::default(), 0.)
                } else {
                    let hero_play_prcnt =
                        wr.total() as f64 / player_total_games[&player_name] as f64;
                    let hero_play_prcnt = (hero_play_prcnt * 1000.).round() / 1000.;
                    (player_name, wr, hero_play_prcnt)
                }
            };
            Some(HeroStats {
                hero_name,
                top_player,
                common_player_raw,
                common_player_relative,
            })
        })
        .collect();
    let players_stats: Vec<(PlayerName, HeroName, WinRatio)> = player_hero_wr
        .iter()
        .map(|(player_name, heroes_wr)| {
            let (hero_name, hero_wr) = match heroes_wr
                .iter()
                .filter(|(_, wr)| wr.total() > 15)
                .max_by_key(|(_, hero_wr)| hero_wr)
            {
                Some(x) => x.clone(),
                None => (String::new(), WinRatio::default()),
            };
            (player_name.clone(), hero_name.clone(), hero_wr.clone())
        })
        .collect();
    HeroPlayersStats {
        heroes_stats,
        players_stats,
    }
}
