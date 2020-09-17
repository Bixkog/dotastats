use serde_json;
use reqwest;
use std::collections::HashSet;
use crate::input_cache::InputCache;

pub type GuildId = String;
pub type PlayerId = String;
pub type MatchId = String;

pub struct DataRetriever {
    cache : InputCache
}

/// Retrieves members steam_id of provided guild via https://api.stratz.com/graphql post endpoint.
/// Request was designed by reverse engineering https://stratz.com/guilds/%guild_id%/members website.
async fn get_guild_members_ids(guild_id : &GuildId) -> reqwest::Result<Vec<PlayerId>> {
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

/// Get match_ids of all games of a specified player. 
/// Uses https://api.opendota.com/api/players/{player_id}/matches endpoint.
async fn get_player_match_ids(player_id : &PlayerId) -> reqwest::Result<Vec<MatchId>> {
    println!("Fetching player matches: {}", player_id);
    let response = reqwest::get(&format!("https://api.opendota.com/api/players/{}/matches", player_id).to_string())
        .await?
        .text()
        .await?;

    let player_matches = serde_json::from_str::<serde_json::Value>(&response).unwrap();
    Ok(player_matches.as_array().unwrap().iter().map(|v| v.as_object().unwrap()["match_id"].as_u64().unwrap().to_string()).collect())
}

/// Get single match data containing parsed match information.
/// Uses https://api.opendota.com/api/matches/{match_id} endpoint.
async fn get_match_info(match_id : &MatchId) -> reqwest::Result<Option<serde_json::Value>> {
    println!("Fetching match info: {}", match_id);
    let response = reqwest::get(&format!("https://api.opendota.com/api/matches/{}", match_id).to_string())
        .await?
        .text()
        .await?;
    let mut res : serde_json::Value = match serde_json::from_str(&response) {
        Ok(v) => v,
        Err(e) => {
            println!("Error {} occured during parsing of {}. Omitting match.", e, response);
            return Ok(None)
        }
    };
    if res["match_id"].is_null() {
        println!("Match_id is missing, assiging: {}", match_id);
        let obj = res.as_object_mut().unwrap();
        obj.insert("match_id".into(), serde_json::Value::Number(match_id.parse::<u64>().unwrap().into()));
        return Ok(Some(serde_json::json!(obj)))
    }
    Ok(Some(res))
}

impl DataRetriever {
    pub fn new() -> DataRetriever {
        DataRetriever{cache : InputCache::new()}
    }

    pub async fn get_guild_players_matches_info(&self, guild_id : &GuildId) -> reqwest::Result<Vec<serde_json::Value>> {
        let members = get_guild_members_ids(guild_id).await?;
        println!("Got {} members of guild: {}", members.len(), &guild_id);
        let mut matches_of_interest = HashSet::new();
        for ref member in members {
            for player_match_id in get_player_match_ids(member).await? {
                matches_of_interest.insert(player_match_id);
            };
        };
        println!("Found {} matches.", matches_of_interest.len());
        let (not_cached_ids, mut cached_infos) = self.cache.get_info(guild_id, &matches_of_interest);
        println!("Cached: {}. Not-cached: {}.", cached_infos.len(), not_cached_ids.len());
        let mut not_cached_info = vec![];
        for not_cached_id in not_cached_ids {
            let new_info = match get_match_info(&not_cached_id).await? {
                Some(info) => info,
                None => continue,
            };
            not_cached_info.push(new_info);
            if not_cached_info.len() > 100 {
                self.cache.add_info(guild_id, &not_cached_info);
                not_cached_info.clear();
            }
        };
        self.cache.add_info(guild_id, &not_cached_info);
        cached_infos.extend(not_cached_info.into_iter());
        Ok(cached_infos)
    }
}