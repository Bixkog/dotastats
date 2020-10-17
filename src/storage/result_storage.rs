use serde::{Deserialize, Serialize};
use chrono::{Utc};
use std::fs::{create_dir, write, read};
use std::io::{Result, ErrorKind};

#[derive(Serialize, Deserialize)]
struct AnalysisResult {
    guild_id: String,
    timestamp: i64,
    payload: serde_json::Value,
}

const ROLES_WR_FILE: &str = "roles_wr.json";

pub fn get_roles_wr_results(guild_id: &String) -> Result<String> {
    let guild_dir = format!("{}_res", guild_id);
    let result = read(format!("{}/{}", guild_dir, ROLES_WR_FILE))?;
    match String::from_utf8(result) {
        Ok(s) => Ok(s),
        Err(e) => Err(std::io::Error::new(ErrorKind::InvalidData, e))
    }
}

pub fn store_roles_wr_result(guild_id: &String, payload: serde_json::Value) -> Result<()> {
    let res = AnalysisResult{
        guild_id: guild_id.clone(), 
        timestamp: Utc::now().timestamp(),
        payload
    };
    let guild_dir = format!("{}_res", guild_id);
    match create_dir(guild_dir.clone()) {
        Ok(()) => (),
        Err(e) => {
            if e.kind() == ErrorKind::AlreadyExists {()}
            else {return Err(e)}
        }
    }
    let res_serialized = serde_json::to_value(res)?.to_string();
    write(format!("{}/{}", guild_dir, ROLES_WR_FILE), res_serialized)?;
    Ok(())
}