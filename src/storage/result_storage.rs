use crate::{storage::Storage, types::GuildId, BoxError};
use chrono::Utc;
use mongodb::options::FindOptions;
use mongodb::{
    self,
    bson::{self, doc},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tokio::stream::StreamExt;

/// Tags specifing which analysis result is stored in payload.
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

/// Analysis results stored in database. Payload is json in raw string.
#[derive(Serialize, Deserialize)]
struct StoredResult {
    guild_id: String,
    timestamp: i64,
    tag: String,
    payload: String,
}

/// Analysis result in format to be sent to client. Payload is parsed.
#[derive(Serialize, Deserialize)]
struct ResultToReturn {
    guild_id: String,
    timestamp: i64,
    payload: serde_json::Value,
}

/// Possible states of analysis results in database.
pub enum ResultsState {
    NotComputed,
    ResultMissing,
    AllComputed { timestamp: i64 },
}
/// Return struct for analysis results state.
pub struct GuildResultsState {
    pub guild_id: GuildId,
    pub state: ResultsState,
}

/// Finds state of guild analysis results in database.
fn extract_results_state(
    guild_id: &GuildId,
    tags_found: &Vec<String>,
    timestamps: &Vec<i64>,
) -> Result<GuildResultsState, BoxError> {
    let all_tags: HashSet<String> = AnalysisTag::iter().map(|tag| tag.to_string()).collect();
    let state: ResultsState = {
        if tags_found.is_empty() {
            ResultsState::NotComputed
        } else if all_tags != tags_found.clone().into_iter().collect() {
            ResultsState::ResultMissing
        } else {
            let last_timestamp = timestamps
                .iter()
                .min()
                .ok_or("Didn't found any timestamps in guild results.")?;
            ResultsState::AllComputed {
                timestamp: last_timestamp.clone(),
            }
        }
    };
    Ok(GuildResultsState {
        guild_id: guild_id.clone(),
        state,
    })
}

impl Storage {
    /// Returns state of calculated analysis results for specified guild.
    pub async fn get_guild_results_state(
        &self,
        guild_id: &GuildId,
    ) -> Result<GuildResultsState, BoxError> {
        let coll = self.db_client.collection("analysis_results");
        let options = FindOptions::builder()
            .projection(doc! {"tag": 1, "timestamp": 1})
            .build();
        let mut cursor = coll.find(doc! {"guild_id": guild_id}, options).await?;
        let mut tags_found = vec![];
        let mut timestamps = vec![];
        while let Some(result_doc) = cursor.next().await {
            let result_doc = result_doc?;
            tags_found.push(result_doc.get_str("tag")?.to_string());
            timestamps.push(result_doc.get_i64("timestamp")?);
        }
        extract_results_state(guild_id, &tags_found, &timestamps)
    }

    /// Returns states of analysis results for each guild. Used by updater.
    pub async fn get_guilds_results_state(&self) -> Result<Vec<GuildResultsState>, BoxError> {
        let coll = self.db_client.collection("analysis_results");
        let options = FindOptions::builder()
            .projection(doc! {"guild_id": 1, "tag": 1, "timestamp": 1})
            .build();
        let mut cursor = coll.find(doc! {}, options).await?;
        let mut tags_found: HashMap<GuildId, Vec<String>> = HashMap::new();
        let mut timestamps: HashMap<GuildId, Vec<i64>> = HashMap::new();
        let mut processed_guilds: HashSet<GuildId> = HashSet::new();
        while let Some(result_doc) = cursor.next().await {
            let result_doc = result_doc?;
            let guild_id = result_doc.get_str("guild_id")?.to_string();
            tags_found
                .entry(guild_id.clone())
                .or_default()
                .push(result_doc.get_str("tag")?.to_string());
            timestamps
                .entry(guild_id.clone())
                .or_default()
                .push(result_doc.get_i64("timestamp")?);
            processed_guilds.insert(guild_id);
        }
        processed_guilds
            .into_iter()
            .map(move |guild_id| {
                extract_results_state(&guild_id, &tags_found[&guild_id], &timestamps[&guild_id])
            })
            .collect()
    }

    /// Stores single analysis result in the database.
    pub async fn store_result(
        &self,
        guild_id: &GuildId,
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
        let coll = self.db_client.collection("analysis_results");
        let filter = doc! {"guild_id": guild_id, "tag": analysis_tag.to_string()};
        coll.delete_one(filter, None).await?;
        coll.insert_one(result_doc, None).await?;
        Ok(())
    }

    /// Retrieves single analysis result from the database in format fiendly for the client.
    pub async fn get_result(
        &self,
        guild_id: &GuildId,
        analysis_tag: AnalysisTag,
    ) -> Result<String, BoxError> {
        let coll = self.db_client.collection("analysis_results");
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
