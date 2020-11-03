use crate::{storage::Storage, BoxError};
use chrono::Utc;
use mongodb::{
    self,
    bson::{self, doc},
};
use serde::de::Error;
use serde::{Deserialize, Serialize};
use serde_json::error::Error as serde_error;
use std::{collections::HashSet, fmt};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tokio::stream::StreamExt;

#[derive(Debug, EnumIter)]
pub enum AnalysisTag {
    RolesWr,
    RolesSynergy,
    RolesRecords,
    HeroesPlayersStats,
    PlayersWr,
}

impl fmt::Display for AnalysisTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize)]
struct StoredResult {
    guild_id: String,
    timestamp: i64,
    tag: String,
    payload: String,
}

#[derive(Serialize, Deserialize)]
struct ResultToReturn {
    guild_id: String,
    timestamp: i64,
    payload: serde_json::Value,
}

impl Storage {
    pub async fn is_guild_stats_ready(&self, guild_id: &String) -> mongodb::error::Result<bool> {
        let db = self.db_client.database("dotastats");
        let coll = db.collection("analysis_results");
        let mut cursor = coll.find(doc! {"guild_id": guild_id}, None).await?;
        let mut tags_found = vec![];
        while let Some(result_doc) = cursor.next().await {
            let result_doc = result_doc?;
            tags_found.push(
                result_doc
                    .get_str("tag")
                    .expect("Field tag not found in analysis result doc.")
                    .to_string(),
            );
        }
        let all_tags: HashSet<String> = AnalysisTag::iter().map(|tag| tag.to_string()).collect();
        Ok(all_tags == tags_found.into_iter().collect())
    }

    pub async fn store_result(
        &self,
        guild_id: &String,
        payload: serde_json::Value,
        analysis_tag: AnalysisTag,
    ) -> mongodb::error::Result<()> {
        let res = StoredResult {
            guild_id: guild_id.clone(),
            timestamp: Utc::now().timestamp(),
            tag: analysis_tag.to_string(),
            payload: payload.to_string(),
        };
        let result_doc = bson::to_document(&res)?;
        let db = self.db_client.database("dotastats");
        let coll = db.collection("analysis_results");
        let filter = doc! {"guild_id": guild_id, "tag": analysis_tag.to_string()};
        coll.delete_one(filter, None).await?;
        coll.insert_one(result_doc, None).await?;
        Ok(())
    }

    pub async fn get_result(
        &self,
        guild_id: &String,
        analysis_tag: AnalysisTag,
    ) -> Result<String, BoxError> {
        let db = self.db_client.database("dotastats");
        let coll = db.collection("analysis_results");
        let filter = doc! {"guild_id": guild_id, "tag": analysis_tag.to_string()};
        let result_doc = coll
            .find_one(filter, None)
            .await?
            .ok_or("File not found.")?;
        let stored_result: StoredResult = bson::from_bson(result_doc.into())?;
        let res = ResultToReturn {
            guild_id: stored_result.guild_id,
            timestamp: stored_result.timestamp,
            payload: serde_json::from_str(stored_result.payload.as_str())?,
        };
        Ok(serde_json::to_string(&res)?)
    }
}
