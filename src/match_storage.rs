use crate::data_retriever::{GuildId, MatchId};
use crate::parse_requester::ParseRequestState;
use crate::utils;
use std::collections::HashSet;

/// Cache used to save match info on hard drive in order not to download it on every run.
/// For each guild creates 2 files: "{guild_id}_ids" which stores saved matches ids
/// and {guild_id}_info which stores matches info.
pub struct MatchStorage {
}

impl MatchStorage {

    pub fn new() -> MatchStorage {
        MatchStorage{}
    }

    fn req_filename(&self, guild_id : &GuildId) -> String {
        format!("{}_req", guild_id)
    }

    fn ids_filename(&self, guild_id : &GuildId) -> String {
        format!("{}_ids", guild_id)
    }

    fn info_filename(&self, guild_id : &GuildId) -> String {
        format!("{}_info", guild_id)
    }

    /// Reads both files, returns ids of matches which were not present in cache and cached match info.
    pub fn get_info<'a>(&self, guild_id : &GuildId, match_ids : &HashSet<MatchId>) -> (Vec<MatchId>, Vec<serde_json::Value>) {
        let fallback_return = (match_ids.clone().into_iter().collect(), vec![]);
        match utils::read_lines(&self.ids_filename(guild_id)) {
            Err(e) => {println!("{}", e); fallback_return}
            Ok(cached_ids) => {
                let not_cached_ids : Vec<MatchId> = match_ids.difference(&cached_ids.into_iter().collect()).map(|p| p.clone()).collect();
                match utils::read_lines(&self.info_filename(guild_id)) {
                    Err(e) => {println!("{}", e); fallback_return}
                    Ok(cached_info) => (not_cached_ids, cached_info.into_iter().map(|s| serde_json::from_str(&s).unwrap()).collect())
                }
            }
        }
    }

    /// Adds ids and info to ids, info guild files.
    pub fn add_info(&self, guild_id : &GuildId, match_infos : &Vec<serde_json::Value>) {
        let match_ids : Vec<String> = match_infos.iter().map(|v| v["match_id"].as_u64().unwrap().to_string()).collect();
        let raw_match_infos = match_infos.iter().map(|v| v.to_string()).collect();
        utils::append_lines(&self.ids_filename(guild_id), &match_ids).expect("Unable to write to guild_ids.");
        utils::append_lines(&self.info_filename(guild_id), &raw_match_infos).expect("Unable to write to guild_info.");
    }

    pub fn get_parse_states(&self, guild_id : &GuildId) -> Vec<ParseRequestState> {
        let requests = utils::read_lines(&self.req_filename(guild_id)).expect("Unable to read requests.");
        requests.iter().map(|s| serde_json::from_str(s).unwrap()).collect()
    }

    pub fn add_parse_states(&self, guild_id : &GuildId, request_states : &Vec<ParseRequestState>) {
        utils::append_lines(&self.req_filename(guild_id), &request_states.iter().map(|v| serde_json::to_string(v).unwrap()).collect())
            .expect("Unable to add parse states.");
    }

    /// Removes records from cache, used to update for parsed matches.
    pub fn set_info(&self, guild_id : &GuildId, match_info : &Vec<serde_json::Value>) {
        utils::clear_file(&self.ids_filename(guild_id)).unwrap();
        utils::clear_file(&self.info_filename(guild_id)).unwrap();
        self.add_info(guild_id, match_info);
    }
}