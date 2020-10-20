use serde::{Deserialize, Serialize};
use chrono::{Utc};
use std::fs::{create_dir, write, read, metadata};
use std::io::{self, ErrorKind};

#[derive(Serialize, Deserialize)]
struct AnalysisResult {
    guild_id: String,
    timestamp: i64,
    payload: serde_json::Value,
}

const ROLES_WR_FILE: &str = "roles_wr.json";
const RESULT_FILES: [&str; 1] = [ROLES_WR_FILE];

pub fn is_guild_result_ready(guild_id: &String) -> io::Result<bool> {
    Ok(RESULT_FILES.iter().all(
        |&filename| 
            match metadata(format!("{}_res/{}", guild_id, filename)) {
                Ok(md) => md.is_file(),
                Err(_) => false
            }
    ))
}

pub fn get_roles_wr_results(guild_id: &String) -> io::Result<String> {
    let result = read(format!("{}_res/{}", guild_id, ROLES_WR_FILE))?;
    match String::from_utf8(result) {
        Ok(s) => Ok(s),
        Err(e) => Err(std::io::Error::new(ErrorKind::InvalidData, e))
    }
}

pub fn store_roles_wr_result(guild_id: &String, payload: serde_json::Value) -> io::Result<()> {
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