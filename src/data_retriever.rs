use crate::match_storage::MatchStorage;
use crate::opendota_client::OpenDotaClient;
use crate::types::{GuildId, MatchId};

use reqwest;
use serde_json;
use std::collections::HashSet;

pub struct DataRetriever {
    storage: MatchStorage,
    od_client: OpenDotaClient,
}

pub struct GuildRawData {
    pub guild_id: GuildId,
    pub members: Vec<serde_json::Value>,
    pub members_matches: Vec<serde_json::Value>,
}

impl DataRetriever {
    pub fn new() -> DataRetriever {
        DataRetriever {
            storage: MatchStorage::new(),
            od_client: OpenDotaClient::new(),
        }
    }

    async fn get_match_data(
        &self,
        guild_id: &GuildId,
        match_ids: Vec<MatchId>,
    ) -> reqwest::Result<Vec<serde_json::Value>> {
        let mut not_cached_info = vec![];
        for match_id in match_ids {
            let new_info = self.od_client.fetch_match_info(&match_id).await?;
            not_cached_info.push(new_info);
            if not_cached_info.len() >= 60 {
                self.storage.add_info(guild_id, &not_cached_info);
                not_cached_info.clear();
            }
        }
        self.storage.add_info(guild_id, &not_cached_info);
        Ok(not_cached_info)
    }

    pub async fn get_guild_raw_data(&self, guild_id: &GuildId) -> reqwest::Result<GuildRawData> {
        let members_ids = self.od_client.fetch_guild_members_ids(guild_id).await?;
        println!("Got {} members of guild: {}", members_ids.len(), &guild_id);
        let mut matches_of_interest = HashSet::new();
        let mut members = vec![];
        for member_id in members_ids.iter() {
            for player_match_id in self.od_client.fetch_player_match_ids(member_id).await? {
                matches_of_interest.insert(player_match_id);
            }
            members.push(self.od_client.fetch_player_info(member_id).await?);
        }
        println!("Found {} matches.", matches_of_interest.len());
        let (not_cached_ids, mut cached_infos) =
            self.storage.get_info(guild_id, &matches_of_interest);
        println!(
            "Cached: {}. Not-cached: {}.",
            cached_infos.len(),
            not_cached_ids.len()
        );
        let not_cached_info = self.get_match_data(guild_id, not_cached_ids).await?;
        cached_infos.extend(not_cached_info.into_iter());
        Ok(GuildRawData {
            guild_id: guild_id.to_string(),
            members,
            members_matches: cached_infos,
        })
    }
}
