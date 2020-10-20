#![feature(proc_macro_hygiene, decl_macro)]
#![feature(try_trait)]
mod analyzers;
mod heroes_info;
mod data_retrieval;
mod match_stats;
mod storage;
mod types;
mod utils;
mod server;
use config::{Config, File};

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
    server::server::run().await?;
    Ok(())
}
