use crate::utils;
use crate::data_retriever::GuildId;
use crate::match_storage::MatchStorage;
use serde::{Deserialize, Serialize};
use reqwest::{ClientBuilder};
use std::collections::HashSet;
use serde_repr::{Serialize_repr, Deserialize_repr};
use tokio::time::{delay_until, Duration, Instant};
// 5424989894
// 5578579202 parsed

pub struct ParseRequester {
    storage : MatchStorage
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum ParseState {
    Sent = 1,
    ReplayMissing = 2,
    Parsed = 3
}

#[derive(Serialize, Deserialize)]
pub struct ParseRequestState {
    pub match_id : String,
    pub state : ParseState
}


type JobId = u64;

impl ParseRequester {

    pub fn new() -> Self {
        ParseRequester{storage : MatchStorage::new()}
    }

    /// Checks whether replay is available on valve servers.
    async fn check_replay_availability(&self, match_json : &serde_json::Value) -> reqwest::Result<bool> {
        let start_inst = Instant::now();
        let request_url = if match_json["replay_url"].is_null() {
            if match_json["cluster"].is_null() || match_json["match_id"].is_null() || match_json["replay_salt"].is_null() {
                return Ok(false)
            }
            format!("http://replay{}.valve.net/570/{}_{}.dem.bz2",
                match_json["cluster"].as_u64().unwrap(),
                match_json["match_id"].as_u64().unwrap(),
                match_json["replay_salt"].as_u64().unwrap())
        } else {match_json["replay_url"].as_str().unwrap().to_string()};

        let client = ClientBuilder::new().build()?;
        let response = client.head(&request_url).send().await?;
        delay_until(start_inst + Duration::from_secs(1)).await;
        Ok(response.status().is_success())
    }

    /// Sends request to open dota to parse certain match.
    async fn send_parse_request(&self, match_id : &String) -> reqwest::Result<()> {
        let start_inst = Instant::now();
        reqwest::Client::new().post(&format!("https://api.opendota.com/api/request/{}", match_id).to_string())
            .header("Content-Length", "0")
            .send()
            .await?
            .text()
            .await?;
        delay_until(start_inst + Duration::from_secs(1)).await;
        Ok(())
    }

    /// Gets current state of match parsing, if it is not parsed it sends request to start.
    async fn get_parse_state(&self, match_json : &serde_json::Value) -> reqwest::Result<ParseRequestState> {
        let replay_present = self.check_replay_availability(&match_json).await?;
        let match_id = match_json["match_id"].as_u64().unwrap().to_string();
        println!("Checking parse state of match: {}", match_id);
        if utils::is_match_parsed(&match_json) {
            println!("Match is parsed.");
            return Ok(ParseRequestState{match_id, state : ParseState::Parsed});
        }
        if replay_present {
            self.send_parse_request(&match_id).await?;
            println!("Match is not parsed. Sent request.");
            return Ok(ParseRequestState{match_id, state : ParseState::Sent});
        } else {
            println!("Match is not parsed. Replay unavailable.");
            return Ok(ParseRequestState{match_id, state : ParseState::ReplayMissing});
        }  
    }

    /// For each match with unknown parsing state, gets its parse state and saves it in storage.
    pub async fn request_parsing(&self, guild_id : &GuildId, match_jsons : &Vec<serde_json::Value>) -> reqwest::Result<()> {
        let old_parse_states : HashSet<String> = self.storage.get_parse_states(guild_id).into_iter().map(|state| state.match_id).collect(); 
        let mut parse_states = vec![];
        for match_json in match_jsons.iter() {
            let match_id = match_json["match_id"].as_u64().unwrap().to_string();
            if !old_parse_states.contains(&match_id) {
                let parse_state = self.get_parse_state(&match_json).await?;
                    parse_states.push(parse_state);
            }
            if parse_states.len() > 60 {
                self.storage.add_parse_states(guild_id, &parse_states);
                parse_states.clear();
            }
        }
        Ok(())
    }
}