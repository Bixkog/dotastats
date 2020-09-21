use crate::match_storage::MatchStorage;
use crate::utils;

use serde_json;
use reqwest;
use std::collections::HashSet;

pub type GuildId = String;
pub type PlayerId = String;
pub type MatchId = String;

pub struct DataRetriever {
    storage: MatchStorage
}

pub struct GuildRawData {
    pub guild_id: GuildId,
    pub members: Vec<serde_json::Value>,
    pub members_matches: Vec<serde_json::Value>
}

/// Retrieves members steam_id of provided guild via https://api.stratz.com/graphql post endpoint.
/// Request was designed by reverse engineering https://stratz.com/guilds/%guild_id%/members website.
async fn fetch_guild_members_ids(guild_id : &GuildId) -> reqwest::Result<Vec<PlayerId>> {
    println!("Fetching members of guild: {}", guild_id);
    let req_body = format!(r#"{{
        "operationName":"GuildMembers",
        "variables":{{"id":{},"byId":true,"tag":"","byTag":false}},"query":"query GuildMembers($id: Int!, $byId: Boolean!, $tag: String!, $byTag: Boolean!) {{\n  guild(id: $id) @include(if: $byId) {{\n    ...GuildMembers\n  }}\n  stratz @include(if: $byTag) {{\n    search(request: {{query: $tag, searchType: [GUILDS]}}) {{\n      guild {{\n        ...GuildMembers\n      }}\n    }}\n  }}\n}}\n\nfragment GuildMembers on GuildType {{\n  members {{\n    steamAccount {{\n      id}}\n  }}\n}}\n"}}"#,
        guild_id);
    let response = reqwest::Client::new()
        .post("https://api.stratz.com/graphql")
        .header("content-type", "application/json")
        .body(req_body)
        .send()
        .await?
        .text()
        .await?;
    let response_json : serde_json::Value = serde_json::from_str(&response).unwrap();
    let members = response_json["data"]["guild"]["members"].as_array().unwrap();
    Ok(members.iter().map(|v| v["steamAccount"]["id"].as_u64().unwrap().to_string()).collect())
}

async fn fetch_player_info(player_id: &PlayerId) -> reqwest::Result<serde_json::Value> {
    println!("Fetching info about player: {}", player_id);
    utils::get_req_at60rpm(&format!("https://api.opendota.com/api/players/{}", player_id).to_string()).await
}

/// Get match_ids of all games of a specified player. 
/// Uses https://api.opendota.com/api/players/{player_id}/matches endpoint.
async fn fetch_player_match_ids(player_id : &PlayerId) -> reqwest::Result<Vec<MatchId>> {
    println!("Fetching player matches: {}", player_id);
    let response = utils::get_req_at60rpm(&format!("https://api.opendota.com/api/players/{}/matches", player_id).to_string()).await?;
    Ok(response.as_array().unwrap().iter().map(|v| v.as_object().unwrap()["match_id"].as_u64().unwrap().to_string()).collect())
}

/// Get single match data containing parsed match information.
/// Uses https://api.opendota.com/api/matches/{match_id} endpoint.
async fn fetch_match_info(match_id : &MatchId) -> reqwest::Result<serde_json::Value> {
    println!("Fetching match info: {}", match_id);
    let mut response = utils::get_req_at60rpm(&format!("https://api.opendota.com/api/matches/{}", match_id).to_string()).await?;
    if response["match_id"].is_null() {
        println!("Match_id is missing, assiging: {}", match_id);
        let obj = response.as_object_mut().unwrap();
        obj.insert("match_id".into(), serde_json::Value::Number(match_id.parse::<u64>().unwrap().into()));
        return Ok(serde_json::json!(obj))
    }
    Ok(response)
}

impl DataRetriever {
    pub fn new() -> DataRetriever {
        DataRetriever{storage : MatchStorage::new()}
    }

    async fn get_match_data(&self, guild_id : &GuildId, match_ids : Vec<MatchId>) -> reqwest::Result<Vec<serde_json::Value>> {
        let mut not_cached_info = vec![];
        for match_id in match_ids {
            let new_info =  fetch_match_info(&match_id).await?;
            not_cached_info.push(new_info);
            if not_cached_info.len() >= 60 {
                self.storage.add_info(guild_id, &not_cached_info);
                not_cached_info.clear();
            }
        };
        self.storage.add_info(guild_id, &not_cached_info);
        Ok(not_cached_info)
    }

    pub async fn get_guild_raw_data(&self, guild_id : &GuildId) -> reqwest::Result<GuildRawData> {
        let members_ids = fetch_guild_members_ids(guild_id).await?;
        println!("Got {} members of guild: {}", members_ids.len(), &guild_id);
        let mut matches_of_interest = HashSet::new();
        let mut members = vec![];
        for member_id in members_ids.iter() {
            for player_match_id in fetch_player_match_ids(member_id).await? {
                matches_of_interest.insert(player_match_id);
            };
            members.push(fetch_player_info(member_id).await?);
        };
        println!("Found {} matches.", matches_of_interest.len());
        let (not_cached_ids, mut cached_infos) = self.storage.get_info(guild_id, &matches_of_interest);
        println!("Cached: {}. Not-cached: {}.", cached_infos.len(), not_cached_ids.len());
        let not_cached_info = self.get_match_data(guild_id, not_cached_ids).await?;
        cached_infos.extend(not_cached_info.into_iter());
        Ok(GuildRawData{guild_id: guild_id.to_string(), members, members_matches: cached_infos})
    }
}