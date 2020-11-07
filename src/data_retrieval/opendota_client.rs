use crate::types::{GuildId, MatchId, PlayerId};
use crate::BoxError;
use reqwest;
use serde::de::Error;
use serde_json::error::Error as serde_error;
use tokio::time::{delay_until, Duration, Instant};

/// Struct which handles all communication with opendota api.
pub struct OpenDotaClient {}

impl OpenDotaClient {
    pub fn new() -> Self {
        Self {}
    }

    /// Sends get requests. Waits 1 second to ensure rpm <= 60.
    async fn get_req_at60rpm(&self, url: &String) -> Result<serde_json::Value, BoxError> {
        let start_inst = Instant::now();
        let response = reqwest::get(url).await?.text().await?;
        delay_until(start_inst + Duration::from_secs(1)).await;
        let parsed_respone = match serde_json::from_str(&response) {
            Ok(json) => json,
            Err(e) => {
                warn!("Unable to parse response at url: {}", url);
                return Err(Box::new(e));
            }
        };
        Ok(parsed_respone)
    }

    /// Retrieves members steam_id of provided guild via https://api.stratz.com/graphql post endpoint.
    /// Request was designed by reverse engineering https://stratz.com/guilds/%guild_id%/members website.
    pub async fn fetch_guild_members_ids(
        &self,
        guild_id: &GuildId,
    ) -> Result<Vec<PlayerId>, BoxError> {
        info!("Fetching members of guild: {}", guild_id);
        let req_body = format!(
            r#"{{
            "operationName":"GuildMembers",
            "variables":{{"id":{},"byId":true,"tag":"","byTag":false}},"query":"query GuildMembers($id: Int!, $byId: Boolean!, $tag: String!, $byTag: Boolean!) {{\n  guild(id: $id) @include(if: $byId) {{\n    ...GuildMembers\n  }}\n  stratz @include(if: $byTag) {{\n    search(request: {{query: $tag, searchType: [GUILDS]}}) {{\n      guild {{\n        ...GuildMembers\n      }}\n    }}\n  }}\n}}\n\nfragment GuildMembers on GuildType {{\n  members {{\n    steamAccount {{\n      id}}\n  }}\n}}\n"}}"#,
            guild_id
        );
        let response = reqwest::Client::new()
            .post("https://api.stratz.com/graphql")
            .header("content-type", "application/json")
            .body(req_body)
            .send()
            .await?
            .text()
            .await?;
        let response_json: serde_json::Value = match serde_json::from_str(&response) {
            Ok(json) => json,
            Err(e) => {
                warn!(
                    "Unable to parse guild members response for guild: {}",
                    guild_id
                );
                return Err(Box::new(e));
            }
        };
        let members = response_json["data"]["guild"]["members"]
            .as_array()
            .ok_or(serde_error::custom("unable to read members of guild"))?;
        Ok(members
            .iter()
            .map(|v| {
                Ok(v["steamAccount"]["id"]
                    .as_u64()
                    .ok_or(serde_error::custom("unable to guild member id"))?
                    .to_string())
            })
            .collect::<Result<Vec<PlayerId>, serde_json::Error>>()?)
    }

    pub async fn fetch_player_info(
        &self,
        player_id: &PlayerId,
    ) -> Result<serde_json::Value, BoxError> {
        info!("Fetching info about player: {}", player_id);
        self.get_req_at60rpm(
            &format!("https://api.opendota.com/api/players/{}", player_id).to_string(),
        )
        .await
    }

    /// Get match_ids of all games of a specified player.
    /// Uses https://api.opendota.com/api/players/{player_id}/matches endpoint.
    pub async fn fetch_player_match_ids(
        &self,
        player_id: &PlayerId,
    ) -> Result<Vec<MatchId>, BoxError> {
        info!("Fetching match ids of player: {}", player_id);
        let response = self
            .get_req_at60rpm(
                &format!("https://api.opendota.com/api/players/{}/matches", player_id).to_string(),
            )
            .await?;
        Ok(response
            .as_array()
            .ok_or(serde_error::custom(
                "unable to parse players match ids respone",
            ))?
            .iter()
            .map(|v| {
                v.as_object().ok_or(serde_error::custom(
                    "players match id response is not object",
                ))?["match_id"]
                    .as_u64()
                    .ok_or(serde_error::custom(
                        "players match id response doesn't contain match_id",
                    ))
            })
            .collect::<Result<Vec<MatchId>, serde_json::Error>>()?)
    }

    /// Get single match data containing parsed match information.
    /// Uses https://api.opendota.com/api/matches/{match_id} endpoint.
    pub async fn fetch_match_info(
        &self,
        match_id: &MatchId,
    ) -> Result<serde_json::Value, BoxError> {
        info!("Fetching match info: {}", match_id);
        let mut response = self
            .get_req_at60rpm(
                &format!("https://api.opendota.com/api/matches/{}", match_id).to_string(),
            )
            .await?;
        if response["match_id"].is_null() {
            warn!("Match_id is missing, assiging: {}", match_id);
            let obj = response.as_object_mut().ok_or(serde_error::custom(
                "unable to cast match json to mutable object",
            ))?;
            obj.insert(
                "match_id".into(),
                serde_json::Value::Number((*match_id as i64).into()),
            );
            return Ok(serde_json::json!(obj));
        }
        Ok(response)
    }
}
