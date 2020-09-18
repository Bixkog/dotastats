use crate::data_retriever::{GuildId, MatchId};
use crate::utils;
use std::collections::HashSet;

/// Cache used to save match info on hard drive in order not to download it on every run.
/// For each guild creates 2 files: "{guild_id}_ids" which stores saved matches ids
/// and {guild_id}_info which stores matches info.
pub struct InputCache {
}

impl InputCache {

    pub fn new() -> InputCache {
        InputCache{}
    }

    /// Reads both files, returns ids of matches which were not present in cache and cached match info.
    pub fn get_info<'a>(&self, guild_id : &GuildId, match_ids : &HashSet<MatchId>) -> (Vec<MatchId>, Vec<serde_json::Value>) {
        let fallback_return = (match_ids.clone().into_iter().collect(), vec![]);
        match utils::read_lines(&format!("{}_ids", guild_id)) {
            Err(e) => {println!("{}", e); fallback_return}
            Ok(cached_ids) => {
                let not_cached_ids : Vec<MatchId> = match_ids.difference(&cached_ids.into_iter().collect()).map(|p| p.clone()).collect();
                match utils::read_lines(&format!("{}_info", guild_id)) {
                    Err(e) => {println!("{}", e); fallback_return}
                    Ok(cached_info) => (not_cached_ids, cached_info.into_iter().map(|s| serde_json::from_str(&s).unwrap()).collect())
                }
            }
        }
    }

    /// Adds ids and info to both guild files.
    pub fn add_info(&self, guild_id : &GuildId, match_infos : &Vec<serde_json::Value>) {
        let match_ids : Vec<String> = match_infos.iter().map(|v| v["match_id"].as_u64().unwrap().to_string()).collect();
        let raw_match_infos = match_infos.iter().map(|v| v.to_string()).collect();
        utils::append_lines(&format!("{}_ids", guild_id), &match_ids).expect("Unable to write to guild_ids.");
        utils::append_lines(&format!("{}_info", guild_id), &raw_match_infos).expect("Unable to write to guild_info.");
    }

    pub fn clear_errors(&self, guild_id : &GuildId) {
        let correct_match_infos : Vec<serde_json::Value> = match utils::read_lines(&format!("{}_info", guild_id)) {
                Err(e) => {println!("Cant open guild_info file for clearing: {}", e); return}
                Ok(cached_info) => cached_info.into_iter().map::<serde_json::Value, _>(|s| serde_json::from_str(&s).unwrap())
                                    .filter(|v| v["error"].is_null()).collect()
            };
        println!("Postclear info count: {}", correct_match_infos.len());
        utils::clear_file(&format!("{}_ids", guild_id)).unwrap();
        utils::clear_file(&format!("{}_info", guild_id)).unwrap();
        self.add_info(guild_id, &correct_match_infos);
    }
}