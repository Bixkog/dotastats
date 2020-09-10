use crate::data_retriever::{GuildId, MatchId};
use std::collections::HashSet;
use std::io::{self, BufRead};

pub struct InputCache {

}

impl InputCache {

    pub fn get_info<'a>(&self, guild_id : &GuildId, match_ids : &'a HashSet<MatchId>) -> (Vec<&'a MatchId>, Vec<serde_json::Value>) {
        match std::fs::File::open(format!("{}_ids", guild_id)) {
            Err(e) => (match_ids.iter().collect(), vec![]),
            Ok(file) => {
                let mut file_ids = HashSet::new();
                for line in io::BufReader::new(file).lines() {
                    match line {
                        Ok(s) => file_ids.insert(s),
                        Err(_) => break
                    
                    };
                };
                let not_cached_ids : Vec<&MatchId> = match_ids.difference(&file_ids).collect();
                // TODO : read {}_info as json array?
                (match_ids.iter().collect(), vec![])
            }
        }
    }

    pub fn add_info(&self, guild_id : &GuildId, match_infos : &Vec<serde_json::Value>) {

    }
}