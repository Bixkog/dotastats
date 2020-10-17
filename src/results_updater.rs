use std::fs::{read_dir};
use regex::Regex;
use std::time::SystemTime;
use crate::request_processing;
use crate::CONFIG;

lazy_static! {
    static ref RE: Regex = Regex::new("([0-9]*)_res").unwrap();
}

fn update_delay_elapsed(dirname: &str) -> std::io::Result<bool> {
    let update_days = CONFIG.get_int("update_every_n_days").unwrap() as u64;
    for file in read_dir(dirname)? {
        let file = file?;
        let last_modified = file.metadata()?.modified()?;
        let now = SystemTime::now();
        let duration_since_modified = now.duration_since(last_modified).unwrap();
        if duration_since_modified.as_secs() > /*day in secs:*/ 86400 * update_days {
            return Ok(true)
        }
    }
    Ok(false)
}

async fn update_guild_data(dirname: &str) -> Result<(), Box<dyn std::error::Error>> {
    let guild_id = RE.captures(dirname).unwrap().get(0).unwrap().as_str().to_string();
    request_processing::process_guild_request(&guild_id, true).await?;
    Ok(())
}

pub async fn update_results() -> Result<(), Box<dyn std::error::Error>> {
    println!("Updating results!.");
    for file in read_dir(".")? {
        let file = file?;
        let os_filename  = file.file_name();
        let filename = os_filename.as_os_str().to_str().unwrap();
        if file.file_type()?.is_dir() && RE.is_match(filename) {
            if update_delay_elapsed(filename)? {
                update_guild_data(filename).await?;

            }
        }
    }
    println!("Update finished!.");

    Ok(())
}