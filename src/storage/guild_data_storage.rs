use crate::storage::Storage;
use crate::types::GuildId;
use crate::CONFIG;
use mongodb::{
    self,
    bson::{self, doc, Bson, Document},
};
use serde::{de::Error, Deserialize, Serialize};
use tokio::stream::StreamExt;

#[derive(Serialize, Deserialize, Clone)]
pub struct MatchData {
    match_id: i64,
    info: String,
}

impl MatchData {
    pub fn from_json(match_data: &serde_json::Value) -> serde_json::error::Result<MatchData> {
        let match_id = match match_data["match_id"].as_i64() {
            Some(id) => id,
            None => {
                return Err(serde_json::error::Error::custom(
                    "No match_id in provided match_data.",
                ))
            }
        };
        Ok(MatchData {
            match_id,
            info: match_data.to_string(),
        })
    }
}

#[derive(Serialize, Deserialize)]
struct GuildDataBatch {
    guild_id: String,
    matches: Vec<MatchData>, // max size: 100
}

fn extract_match_data_from_doc(match_bson: &mut Bson) -> Option<serde_json::Value> {
    let match_json: serde_json::Value = match_bson.clone().into();
    match serde_json::from_str(match_json["info"].as_str()?) {
        Ok(json) => Some(json),
        Err(e) => {
            warn!("Can't parse match data: {}", e);
            None
        }
    }
}

impl Storage {
    /// Retrieves matches of a guild from dotastats/guild_data collection.
    pub async fn get_guild_data(
        &self,
        guild_id: &GuildId,
    ) -> mongodb::error::Result<Vec<serde_json::Value>> {
        let db = self.db_client.database("dotastats");
        let coll = db.collection("guild_data");
        let filter = doc! {"guild_id": guild_id};
        let mut cursor = coll.find(filter, None).await?;
        let mut res = vec![];
        while let Some(guild_batch_doc) = cursor.next().await {
            for match_bsons in guild_batch_doc?.get_array_mut("matches") {
                let mut match_json_batch: Vec<serde_json::Value> = match_bsons
                    .into_iter()
                    .filter_map(extract_match_data_from_doc)
                    .collect();
                res.append(&mut match_json_batch);
            }
        }
        Ok(res)
    }

    /// Adds match data to dotastats/guild_data collection, in 100 elements batches.
    pub async fn add_guild_data(
        &self,
        guild_id: &GuildId,
        match_data: &Vec<serde_json::Value>,
    ) -> mongodb::error::Result<()> {
        if match_data.is_empty() {
            return Ok(());
        }
        let match_docs: Vec<MatchData> = match_data
            .iter()
            .filter_map(|match_json| match MatchData::from_json(match_json) {
                Ok(res) => Some(res),
                Err(e) => {
                    error!(
                        "Unable to prepare match json to save to the database: {}",
                        e
                    );
                    None
                }
            })
            .collect();
        let chunk_size = CONFIG
            .get_int("db_guild_data_chunk_size")
            .expect("Field db_guild_data_chunk_size not set in config.")
            as usize;
        let guild_data_batches: Vec<Document> = match_docs
            .chunks(chunk_size)
            .map(|c| c.to_vec())
            .into_iter()
            .map(|md| GuildDataBatch {
                guild_id: guild_id.clone(),
                matches: md,
            })
            .filter_map(|gdb| match bson::to_document(&gdb) {
                Ok(res) => Some(res),
                Err(e) => {
                    info!("Unable to convert GuildDataBatch to bson::Document: {}", e);
                    None
                }
            })
            .collect();
        let db = self.db_client.database("dotastats");
        let coll = db.collection("guild_data");
        coll.insert_many(guild_data_batches, None).await?;
        Ok(())
    }
}
