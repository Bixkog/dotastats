#![feature(proc_macro_hygiene, decl_macro)]
#![feature(try_trait)]
mod analyzers;
mod data_retrieval;
mod heroes_info;
mod match_stats;
mod server;
mod storage;
mod types;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate log;
extern crate scanpw;
extern crate simplelog;

use config;
use simplelog::*;
use std::fs::File;

pub type BoxError = Box<dyn std::error::Error + std::marker::Send + std::marker::Sync>;

lazy_static! {
    pub static ref CONFIG: config::Config = {
        let mut config = config::Config::default();
        config
            .merge(config::File::with_name("config.json"))
            .unwrap();
        config
    };
}

fn init_logging() {
    let logging_config = ConfigBuilder::new()
        .set_location_level(LevelFilter::Warn)
        .build();
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            logging_config.clone(),
            TerminalMode::Mixed,
        )
        .unwrap(),
        WriteLogger::new(
            LevelFilter::Info,
            logging_config.clone(),
            File::create("server.log").unwrap(),
        ),
    ])
    .unwrap();
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    init_logging();
    server::server::run().await?;
    Ok(())
}
