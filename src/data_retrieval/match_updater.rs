use crate::data_retrieval::opendota_client::OpenDotaClient;
use crate::data_retrieval::parse_requester::{ParseRequestState, ParseState};
use crate::storage::match_storage::MatchStorage;
use crate::types::{GuildId, MatchId};
use crate::utils::is_match_parsed;
use std::collections::HashMap;

pub struct MatchUpdater {
    storage: MatchStorage,
    od_client: OpenDotaClient,
}

impl MatchUpdater {
    pub fn new() -> Self {
        MatchUpdater {
            storage: MatchStorage::new(),
            od_client: OpenDotaClient::new(),
        }
    }

    /// Returns only fully parsed match.
    async fn get_parsed_match(
        &self,
        match_id: &MatchId,
    ) -> reqwest::Result<Option<serde_json::Value>> {
        let match_info = self.od_client.fetch_match_info(match_id).await?;
        if is_match_parsed(&match_info) {
            println!("Updated match: {}", match_id);
            Ok(Some(match_info))
        } else {
            Ok(None)
        }
    }

    /// Changes storage to contain updated matches.
    fn update(&self, guild_id: &GuildId, parsed_matches: Vec<serde_json::Value>) {
        let mut saved_matches = self.storage.get_match_data(guild_id);
        let mut parsed_matches_map: HashMap<MatchId, serde_json::Value> = parsed_matches
            .into_iter()
            .map(|v| (v["match_id"].as_u64().unwrap(), v))
            .collect();
        for saved_match in saved_matches.iter_mut() {
            let match_id = saved_match["match_id"].as_u64().unwrap();
            if parsed_matches_map.contains_key(&match_id) {
                let (_, parsed_match) = parsed_matches_map.remove_entry(&match_id).unwrap();
                *saved_match = parsed_match;
            }
        }
        self.storage.set_info(guild_id, &saved_matches);
    }

    /// Reads sent parse requests, and checks whether matches are already parsed.
    /// Updates storage to contain newly parsed matches.
    pub async fn try_update(&self, guild_id: &GuildId) -> reqwest::Result<()> {
        let mut parse_states = self.storage.get_parse_states(guild_id);
        let mut sent_requests: Vec<&mut ParseRequestState> = parse_states
            .iter_mut()
            .filter(|p| p.state == ParseState::Sent)
            .collect();
        println!("Trying to update {} matches.", sent_requests.len());
        let mut new_parsed_matches = vec![];
        for sent_request in sent_requests.iter_mut() {
            let parsed_match = match self.get_parsed_match(&sent_request.match_id).await? {
                Some(v) => v,
                None => continue,
            };
            sent_request.state = ParseState::Parsed;
            new_parsed_matches.push(parsed_match);
        }
        if new_parsed_matches.len() == 0 {
            println!("No matches to update.");
            return Ok(());
        }
        println!("Updating {} matches.", new_parsed_matches.len());
        self.update(guild_id, new_parsed_matches);
        self.storage.update_parse_states(guild_id, &parse_states);
        Ok(())
    }
}
