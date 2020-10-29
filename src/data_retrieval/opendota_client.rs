use crate::types::{GuildId, MatchId, PlayerId};
use reqwest;
use tokio::time::{delay_until, Duration, Instant};

pub struct OpenDotaClient {}

impl OpenDotaClient {
    pub fn new() -> Self {
        Self {}
    }
    /// Sends get requests. Waits 1 second to ensure rpm <= 60.
    async fn get_req_at60rpm(&self, url: &String) -> reqwest::Result<serde_json::Value> {
        let start_inst = Instant::now();
        let response = reqwest::get(url).await?.text().await?;
        delay_until(start_inst + Duration::from_secs(1)).await;
        Ok(serde_json::from_str(&response)
            .expect(format!("Can't parse response: {}", response).as_str()))
    }

    /// Retrieves members steam_id of provided guild via https://api.stratz.com/graphql post endpoint.
    /// Request was designed by reverse engineering https://stratz.com/guilds/%guild_id%/members website.
    pub async fn fetch_guild_members_ids(
        &self,
        guild_id: &GuildId,
    ) -> reqwest::Result<Vec<PlayerId>> {
        println!("Fetching members of guild: {}", guild_id);
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
        let response_json: serde_json::Value = serde_json::from_str(&response).unwrap();
        let members = response_json["data"]["guild"]["members"]
            .as_array()
            .unwrap();
        Ok(members
            .iter()
            .map(|v| v["steamAccount"]["id"].as_u64().unwrap().to_string())
            .collect())
    }

    pub async fn fetch_player_info(
        &self,
        player_id: &PlayerId,
    ) -> reqwest::Result<serde_json::Value> {
        println!("Fetching info about player: {}", player_id);
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
    ) -> reqwest::Result<Vec<MatchId>> {
        println!("Fetching player matches: {}", player_id);
        let response = self
            .get_req_at60rpm(
                &format!("https://api.opendota.com/api/players/{}/matches", player_id).to_string(),
            )
            .await?;
        Ok(response
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_object().unwrap()["match_id"].as_u64().unwrap())
            .collect())
    }

    /// Get single match data containing parsed match information.
    /// Uses https://api.opendota.com/api/matches/{match_id} endpoint.
    pub async fn fetch_match_info(&self, match_id: &MatchId) -> reqwest::Result<serde_json::Value> {
        println!("Fetching match info: {}", match_id);
        let mut response = self
            .get_req_at60rpm(
                &format!("https://api.opendota.com/api/matches/{}", match_id).to_string(),
            )
            .await?;
        if response["match_id"].is_null() {
            println!("Match_id is missing, assiging: {}", match_id);
            let obj = response.as_object_mut().unwrap();
            obj.insert(
                "match_id".into(),
                serde_json::Value::Number((*match_id as i64).into()),
            );
            return Ok(serde_json::json!(obj));
        }
        Ok(response)
    }
}
