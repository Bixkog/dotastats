use crate::data_retriever::GuildRawData;
use serde::{Deserialize, Serialize};
use serde::ser::Error;
use serde_json::Result;

/// Struct representing players stats at some match.
#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerStats {
    #[serde(default)]
    personaname: Option<String>,
    #[serde(default)]
    hero_id: Option<u64>,
    #[serde(default)]
    win: Option<u64>,
    // kda stats
    #[serde(default)]
    kda: Option<f64>,
    #[serde(default)]
    kills: Option<u64>,
    #[serde(default)]
    deaths: Option<u64>,
    #[serde(default)]
    assists: Option<u64>,
    // exp/gold stats
    #[serde(default)]
    gold_per_min: Option<u64>,
    #[serde(default)]
    xp_per_min: Option<u64>,
    #[serde(default)]
    level: Option<u64>,
    // laning stats
    #[serde(default)]
    lane_kills: Option<u64>,
    #[serde(default)]
    lane: Option<u64>,
    #[serde(default)]
    lane_role: Option<u64>,
    #[serde(default)]
    is_roaming: Option<bool>,
    #[serde(default)]
    last_hits: Option<u64>,
    #[serde(default)]
    denies: Option<u64>,
    #[serde(default)]
    neutral_kills: Option<u64>,
    #[serde(default)]
    lane_efficiency: Option<f64>,
    //fighting stats
    #[serde(default)]
    hero_damage: Option<u64>,
    #[serde(default)]
    hero_healing: Option<u64>,
    #[serde(default)]
    stuns: Option<f64>, // Total stun duration of all stuns by the player
    //support stats
    #[serde(default)]
    camps_stacked: Option<u64>,
    #[serde(default)]
    creeps_stacked: Option<u64>,
    #[serde(default)]
    obs_placed: Option<u64>,
    #[serde(default)]
    sen_placed: Option<u64>,
    #[serde(default)]
    observer_kills: Option<u64>,
    #[serde(default)]
    sen_kills: Option<u64>,
    // pushing stats
    #[serde(default)]
    tower_damage: Option<u64>,
    #[serde(default)]
    tower_kills: Option<u64>,
    //misc stats
    #[serde(default)]
    purchase_tpscroll: Option<u64>,
    #[serde(default)]
    buyback_count: Option<u64>,
    #[serde(default)]
    courier_kills: Option<u64>,
    #[serde(default)]
    rune_pickups: Option<u64>,
    #[serde(default)]
    roshan_kills: Option<u64>,
}

/// Struct containing match global stats.
#[derive(Serialize, Deserialize, Debug)]
pub struct MatchStats {
    #[serde(default)]
    skill: Option<u64>, // Normal, High, Very High
}

type MemberName = String;

#[derive(Serialize, Deserialize, Debug)]
pub struct Match {
    match_stats: MatchStats,
    players_stats: Vec<PlayerStats>,
}

fn extract_match_stats(member_names: &Vec<MemberName>, match_raw_data: serde_json::Value) -> Result<Match> {
    if match_raw_data["players"].is_null() {return Err(serde_json::Error::custom("No players data."))}
    let match_players : Vec<serde_json::Value> = match_raw_data["players"].as_array().unwrap().clone().to_vec();
    let mut players_stats = vec![];
    for player in match_players {
        if player["personaname"].is_null() {continue};
        if member_names.contains(&player["personaname"].as_str().unwrap().to_string()) {
            let player_stats: PlayerStats = serde_json::from_value(player)?;
            players_stats.push(player_stats);
        }
    }
    let match_stats: MatchStats = serde_json::from_value(match_raw_data)?;
    Ok(Match{match_stats, players_stats})
}

pub fn extract_stats(guild_raw_data : GuildRawData) -> Result<Vec<Match>> {
    let member_names : Vec<MemberName> = guild_raw_data.members.iter()
        .filter(|member| !member["profile"]["personaname"].is_null()) 
        .map(|member| member["profile"]["personaname"].as_str().unwrap().to_string()).collect();
    let mut matches_stats = vec![];
    let mut parsed = 0;
    let total = guild_raw_data.members_matches.len();
    for match_raw_data in guild_raw_data.members_matches {
        let match_stats = match extract_match_stats(&member_names, match_raw_data.clone()) {
            Ok(m) => {parsed += 1; m}
            Err(_) => continue
        };
        matches_stats.push(match_stats);
    }
    println!("Parsed {} out of {} matches", parsed, total);
    Ok(matches_stats)
}

#[cfg(test)]
mod tests {
    
    #[test]
    fn extract_match_stats_doesnt_fail() {
        let test_match_json = r#"{
                                "barracks_status_dire":51,
                                "barracks_status_radiant":63,
                                "chat":null,
                                "cluster":133,
                                "cosmetics":null,
                                "dire_score":29,
                                "dire_team_id":null,
                                "draft_timings":null,
                                "duration":2056,
                                "engine":1,
                                "first_blood_time":134,
                                "game_mode":22,
                                "human_players":10,
                                "leagueid":0,
                                "lobby_type":0,
                                "match_id":5583392128,
                                "match_seq_num":4687235745,
                                "negative_votes":0,
                                "objectives":null,
                                "patch":46,
                                "picks_bans":[],
                                "players":[{
                                    "abandons":0,
                                    "ability_targets":null,
                                    "ability_upgrades_arr":[5218,
                                    5220,
                                    5218,
                                    5219,
                                    5218,
                                    5221,
                                    5218,
                                    5220,
                                    5220,
                                    5906,
                                    5220,
                                    5221,
                                    5219,
                                    5219,
                                    5943,
                                    5219,
                                    5221,
                                    5942],
                                    "ability_uses":null,
                                    "account_id":107395529,
                                    "actions":null,
                                    "additional_units":null,
                                    "assists":8,
                                    "backpack_0":0,
                                    "backpack_1":0,
                                    "backpack_2":0,
                                    "backpack_3":null,
                                    "benchmarks":{"gold_per_min":{"pct":0.651270207852194,
                                    "raw":508},
                                    "hero_damage_per_min":{"pct":0.9803695150115472,
                                    "raw":1234.5525291828794},
                                    "hero_healing_per_min":{"pct":0.7632794457274826,
                                    "raw":0},
                                    "kills_per_min":{"pct":0.9711316397228638,
                                    "raw":0.4961089494163424},
                                    "last_hits_per_min":{"pct":0.44803695150115475,
                                    "raw":4.698443579766537},
                                    "lhten":{},
                                    "stuns_per_min":{"pct":0.7182448036951501,
                                    "raw":0},
                                    "tower_damage":{"pct":0.43071593533487296,
                                    "raw":1662},
                                    "xp_per_min":{"pct":0.7424942263279446,
                                    "raw":691}},
                                    "buyback_log":null,
                                    "camps_stacked":null,
                                    "cluster":133,
                                    "connection_log":null,
                                    "cosmetics":[],
                                    "creeps_stacked":null,
                                    "damage":null,
                                    "damage_inflictor":null,
                                    "damage_inflictor_received":null,
                                    "damage_taken":null,
                                    "damage_targets":null,
                                    "deaths":6,
                                    "denies":12,
                                    "dn_t":null,
                                    "duration":2056,
                                    "firstblood_claimed":null,
                                    "game_mode":22,
                                    "gold":202,
                                    "gold_per_min":508,
                                    "gold_reasons":null,
                                    "gold_spent":16695,
                                    "gold_t":null,
                                    "hero_damage":42304,
                                    "hero_healing":0,
                                    "hero_hits":null,
                                    "hero_id":47,
                                    "isRadiant":false,
                                    "is_contributor":false,
                                    "item_0":158,
                                    "item_1":206,
                                    "item_2":263,
                                    "item_3":0,
                                    "item_4":75,
                                    "item_5":63,
                                    "item_neutral":212,
                                    "item_uses":null,
                                    "kda":3,
                                    "kill_streaks":null,
                                    "killed":null,
                                    "killed_by":null,
                                    "kills":17,
                                    "kills_log":null,
                                    "kills_per_min":0.4961089494163424,
                                    "lane_pos":null,
                                    "last_hits":161,
                                    "last_login":null,
                                    "leaver_status":0,
                                    "level":23,
                                    "lh_t":null,
                                    "life_state":null,
                                    "lobby_type":0,
                                    "lose":1,
                                    "match_id":5583392128,
                                    "max_hero_hit":null,
                                    "multi_kills":null,
                                    "name":null,
                                    "obs":null,
                                    "obs_left_log":null,
                                    "obs_log":null,
                                    "obs_placed":null,
                                    "party_id":4,
                                    "party_size":3,
                                    "patch":46,
                                    "performance_others":null,
                                    "permanent_buffs":null,
                                    "personaname":"muhah",
                                    "pings":null,
                                    "player_slot":129,
                                    "pred_vict":null,
                                    "purchase":null,
                                    "purchase_log":null,
                                    "radiant_win":true,
                                    "randomed":null,
                                    "rank_tier":null,
                                    "region":3,
                                    "repicked":null,
                                    "roshans_killed":null,
                                    "rune_pickups":null,
                                    "runes":null,
                                    "runes_log":null,
                                    "sen":null,
                                    "sen_left_log":null,
                                    "sen_log":null,
                                    "sen_placed":null,
                                    "start_time":1598297422,
                                    "stuns":null,
                                    "teamfight_participation":null,
                                    "times":null,
                                    "total_gold":17407,
                                    "total_xp":23678,
                                    "tower_damage":1662,
                                    "towers_killed":null,
                                    "win":0,
                                    "xp_per_min":691,
                                    "xp_reasons":null,
                                    "xp_t":null
                                }],
                                "positive_votes":0,
                                "radiant_gold_adv":null,
                                "radiant_score":34,
                                "radiant_team_id":null,
                                "radiant_win":true,
                                "radiant_xp_adv":null,
                                "region":3,
                                "replay_salt":717057539,
                                "replay_url":"http://replay133.valve.net/570/5583392128_717057539.dem.bz2",
                                "series_id":0,
                                "series_type":0,
                                "skill":1,
                                "start_time":1598297422,
                                "teamfights":null,
                                "tower_status_dire":390,
                                "tower_status_radiant":1968,
                                "version":null}"#;
        super::extract_match_stats(&vec!["muhah".to_string()], serde_json::from_str(test_match_json).unwrap()).unwrap();
    }
}
