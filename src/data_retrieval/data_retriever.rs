use crate::data_retrieval::opendota_client::OpenDotaClient;
use crate::storage::Storage;
use crate::types::{GuildId, MatchId};
use crate::CONFIG;
use serde_json;
use std::collections::HashSet;
use std::sync::Arc;

pub struct DataRetriever {
    storage: Arc<Storage>,
    od_client: OpenDotaClient,
}

pub struct GuildRawData {
    pub guild_id: GuildId,
    pub members: Vec<serde_json::Value>,
    pub members_matches: Vec<serde_json::Value>,
}

impl DataRetriever {
    pub fn new(storage: Arc<Storage>) -> DataRetriever {
        DataRetriever {
            storage,
            od_client: OpenDotaClient::new(),
        }
    }

    /// Downloads missing matches data. In case of crash, data is saved every 100 records.
    async fn get_match_data(
        &self,
        guild_id: &GuildId,
        match_ids: Vec<MatchId>,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        let chunk_size = CONFIG
            .get_int("db_guild_data_chunk_size")
            .expect("Field db_guild_data_chunk_size not set in config.")
            as usize;
        let mut not_cached_info = vec![];
        for match_id in match_ids {
            let new_info = self.od_client.fetch_match_info(&match_id).await?;
            not_cached_info.push(new_info);
            if not_cached_info.len() >= chunk_size {
                self.storage
                    .add_guild_data(guild_id, &not_cached_info)
                    .await?;
                not_cached_info.clear();
            }
        }
        self.storage
            .add_guild_data(guild_id, &not_cached_info)
            .await?;
        Ok(not_cached_info)
    }

    /// Gets match data for guild, either from db or opendota. Also saves missing match data to the db.
    pub async fn get_guild_raw_data(
        &self,
        guild_id: &GuildId,
    ) -> Result<GuildRawData, Box<dyn std::error::Error>> {
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

        let mut cached_matches = self.storage.get_guild_data(guild_id).await?;
        let cached_ids: HashSet<MatchId> = cached_matches
            .iter()
            .filter_map(|match_| match_["match_id"].as_i64())
            .map(|id| id as u64)
            .collect();
        let not_cached_ids: Vec<MatchId> = matches_of_interest
            .difference(&cached_ids)
            .map(|id| id.clone())
            .collect();
        println!(
            "Cached: {}. Not-cached: {}.",
            cached_ids.len(),
            not_cached_ids.len()
        );
        let not_cached_info = self.get_match_data(guild_id, not_cached_ids).await?;
        cached_matches.extend(not_cached_info.into_iter());
        Ok(GuildRawData {
            guild_id: guild_id.to_string(),
            members,
            members_matches: cached_matches,
        })
    }
}
