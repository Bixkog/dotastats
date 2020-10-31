use crate::storage::Storage;
use chrono::Utc;
use mongodb::{
    self,
    bson::{self, doc},
};
use serde::{Deserialize, Serialize};
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
struct AnalysisResult {
    guild_id: String,
    timestamp: i64,
    tag: String,
    payload: String,
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
        let res = AnalysisResult {
            guild_id: guild_id.clone(),
            timestamp: Utc::now().timestamp(),
            tag: analysis_tag.to_string(),
            payload: payload.to_string(),
        };
        let result_doc = bson::to_document(&res)?;
        let db = self.db_client.database("dotastats");
        let coll = db.collection("analysis_results");
        coll.insert_one(result_doc, None).await?;
        Ok(())
    }

    pub async fn get_result(
        &self,
        guild_id: &String,
        analysis_tag: AnalysisTag,
    ) -> mongodb::error::Result<Option<String>> {
        let db = self.db_client.database("dotastats");
        let coll = db.collection("analysis_results");
        let filter = doc! {"guild_id": guild_id, "tag": analysis_tag.to_string()};
        let result_doc = coll.find_one(filter, None).await?;
        match result_doc {
            Some(result_doc) => Ok(Some(
                result_doc
                    .get_str("payload")
                    .expect("Field payload not present in result document.")
                    .to_string(),
            )),
            None => Ok(None),
        }
    }
}
