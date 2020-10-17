#![feature(proc_macro_hygiene, decl_macro)]
#![feature(try_trait)]
mod analyzers;
mod heroes_info;
mod data_retrieval;
mod match_stats;
mod storage;
mod types;
mod utils;
mod request_processing;
mod server;
mod results_updater;
use config::{Config, File};
use std::time::Duration;
#[macro_use]
extern crate lazy_static;
#[macro_use] 
extern crate rocket;

lazy_static! {
    pub static ref CONFIG: Config = {
        let mut config = Config::default();
        config.merge(File::with_name("config.json")).unwrap();
        config
    };
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let updater = tokio::spawn(async {
        loop {
            match results_updater::update_results().await {
                Ok(()) => (),
                Err(e) => {
                    println!("Error {} occured during update. Retry in 1 hour.", e);
                    ()
                }
            };
            std::thread::sleep(Duration::from_secs(3600));
        }
    });
    server::start().await?;
    updater.await?;
    Ok(())
}
