use crate::server::data_processing;
use crate::CONFIG;
use regex::Regex;
use std::fs::read_dir;
use std::time::SystemTime;
use tokio::task::JoinHandle;

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
            return Ok(true);
        }
    }
    Ok(false)
}

async fn update_guild_data(
    data_processing_queue: data_processing::DPQ,
    dirname: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let guild_id = RE
        .captures(dirname)
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .to_string();
    println!("Adding guild {} to update queue.", guild_id);
    data_processing_queue
        .write()
        .await
        .push_back((guild_id, true));
    Ok(())
}

async fn update_results(
    data_processing_queue: data_processing::DPQ,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Updating results!.");
    for file in read_dir(".")? {
        let file = file?;
        let os_filename = file.file_name();
        let filename = os_filename.as_os_str().to_str().unwrap();
        if file.file_type()?.is_dir() && RE.is_match(filename) {
            println!("Dir name matched.");
            if update_delay_elapsed(filename)? {
                update_guild_data(data_processing_queue.clone(), filename).await?;
            }
        }
    }
    println!("Update finished!.");
    Ok(())
}

pub async fn spawn_worker(data_processing_queue: data_processing::DPQ) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match update_results(data_processing_queue.clone()).await {
                Ok(()) => (),
                Err(e) => {
                    println!("Error {} occured during update. Retry in 1 hour.", e);
                    ()
                }
            };
            tokio::time::delay_for(tokio::time::Duration::from_secs(3600)).await;
        }
    })
}
