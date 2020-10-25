use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{create_dir, metadata, read, write};
use std::io::{self, ErrorKind};

#[derive(Serialize, Deserialize)]
struct AnalysisResult {
    guild_id: String,
    timestamp: i64,
    payload: serde_json::Value,
}

const ROLES_WR_FILE: &str = "roles_wr.json";
const ROLES_SYNERGY_FILE: &str = "roles_synergy.json";
const ROLES_RECORDS_FILE: &str = "roles_records.json";
const HEROES_PLAYERS_STATS_FILE: &str = "heroes_players_stats.json";
const RESULT_FILES: [&str; 4] = [
    ROLES_WR_FILE,
    ROLES_SYNERGY_FILE,
    ROLES_RECORDS_FILE,
    HEROES_PLAYERS_STATS_FILE,
];

pub fn is_guild_result_ready(guild_id: &String) -> io::Result<bool> {
    Ok(RESULT_FILES.iter().all(|&filename| {
        match metadata(format!("{}_res/{}", guild_id, filename)) {
            Ok(md) => md.is_file(),
            Err(_) => false,
        }
    }))
}

fn create_guild_dir(guild_id: &String) -> io::Result<()> {
    let guild_dir = format!("{}_res", guild_id);
    match create_dir(guild_dir.clone()) {
        Ok(()) => Ok(()),
        Err(e) => {
            if e.kind() == ErrorKind::AlreadyExists {
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}

fn store_result(
    guild_id: &String,
    payload: serde_json::Value,
    target_filename: &str,
) -> io::Result<()> {
    let res = AnalysisResult {
        guild_id: guild_id.clone(),
        timestamp: Utc::now().timestamp(),
        payload,
    };
    create_guild_dir(guild_id)?;
    let res_serialized = serde_json::to_value(res)?.to_string();
    write(
        format!("{}_res/{}", guild_id, target_filename),
        res_serialized,
    )?;
    Ok(())
}

fn get_results(guild_id: &String, target_filename: &str) -> io::Result<String> {
    let result = read(format!("{}_res/{}", guild_id, target_filename))?;
    match String::from_utf8(result) {
        Ok(s) => Ok(s),
        Err(e) => Err(std::io::Error::new(ErrorKind::InvalidData, e)),
    }
}

pub fn store_roles_wr_result(guild_id: &String, payload: serde_json::Value) -> io::Result<()> {
    store_result(guild_id, payload, ROLES_WR_FILE)
}

pub fn store_roles_synergy_result(guild_id: &String, payload: serde_json::Value) -> io::Result<()> {
    store_result(guild_id, payload, ROLES_SYNERGY_FILE)
}

pub fn store_roles_records_result(guild_id: &String, payload: serde_json::Value) -> io::Result<()> {
    store_result(guild_id, payload, ROLES_RECORDS_FILE)
}

pub fn store_heroes_players_stats_result(
    guild_id: &String,
    payload: serde_json::Value,
) -> io::Result<()> {
    store_result(guild_id, payload, HEROES_PLAYERS_STATS_FILE)
}

pub fn get_roles_wr_results(guild_id: &String) -> io::Result<String> {
    get_results(guild_id, ROLES_WR_FILE)
}

pub fn get_roles_synergy_results(guild_id: &String) -> io::Result<String> {
    get_results(guild_id, ROLES_SYNERGY_FILE)
}

pub fn get_roles_records_results(guild_id: &String) -> io::Result<String> {
    get_results(guild_id, ROLES_RECORDS_FILE)
}

pub fn get_heroes_players_stats_results(guild_id: &String) -> io::Result<String> {
    get_results(guild_id, HEROES_PLAYERS_STATS_FILE)
}
