use crate::data_retrieval::data_retriever::GuildRawData;
use crate::match_stats::{Match, MatchStats, PlayerStats};

use serde::de::Error;
use serde_json::error::Error as serde_error;
use serde_json::Result;

use std::collections::HashMap;

type MemberName = String;

/// Extracts single match stats, including only guild members information.
fn extract_match_stats(
    member_names: &Vec<MemberName>,
    match_json: serde_json::Value,
) -> Result<Match> {
    let match_players: &Vec<serde_json::Value> = match_json["players"]
        .as_array()
        .ok_or(serde_error::custom("No players data."))?;
    let players_stats = match_players
        .iter()
        .filter(|player| player["personaname"].is_string())
        .filter(|player| {
            member_names.contains(&player["personaname"].as_str().unwrap().to_string())
        })
        .map(|player| serde_json::from_value(player.clone()))
        .collect::<Result<Vec<PlayerStats>>>()?;
    let match_stats: MatchStats = serde_json::from_value(match_json)?;
    Ok(Match::new(match_stats, players_stats))
}

/// Extracts specific match data from dota open api match json's.
pub fn extract_stats(guild_raw_data: GuildRawData) -> Result<Vec<Match>> {
    let member_names: Vec<MemberName> = guild_raw_data
        .members
        .iter()
        .filter(|member| member["profile"]["personaname"].is_string())
        .map(|member| {
            member["profile"]["personaname"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect();
    let mut matches_stats = vec![];
    let mut parsed = 0;
    let total = guild_raw_data.members_matches.len();
    let mut parsed_team_size = [0; 5];
    for match_json in guild_raw_data.members_matches {
        let match_stats = match extract_match_stats(&member_names, match_json.clone()) {
            Ok(m) => {
                parsed += 1;
                m
            }
            Err(_) => continue,
        };
        parsed_team_size[match_stats.get_team_size() - 1] += 1;
        matches_stats.push(match_stats);
    }
    info!("Parsed team sizes: {:?}", parsed_team_size);
    info!("Parsed {} out of {} matches", parsed, total);
    let parsing_stats = compute_parsing_stats(&matches_stats);
    info!("{:#?}", parsing_stats);
    Ok(matches_stats)
}

/// Counts fields occurences. Used to check whether some field occurs frequent enough to be useful.
fn compute_parsing_stats(matches: &Vec<Match>) -> HashMap<String, u32> {
    let mut field_cnt: HashMap<String, u32> = HashMap::new();
    fn update_field(field_cnt: &mut HashMap<String, u32>, k: String, v: serde_json::Value) {
        match v {
            serde_json::Value::Null => return,
            serde_json::Value::Object(map) => {
                for (k_, v_) in map.into_iter() {
                    update_field(field_cnt, k_, v_);
                }
            }
            serde_json::Value::Array(arr) => {
                for e in arr {
                    update_field(field_cnt, k.clone(), e)
                }
            }
            _ => *field_cnt.entry(k).or_insert(0) += 1,
        }
    };

    for match_ in matches.iter() {
        update_field(
            &mut field_cnt,
            String::new(),
            serde_json::to_value(match_).unwrap(),
        );
    }
    field_cnt
}
