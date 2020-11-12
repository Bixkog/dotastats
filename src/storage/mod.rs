pub mod guild_data_storage;
pub mod result_storage;
use crate::BoxError;
use crate::CONFIG;
use mongodb::bson::doc;
use scanpw;
use serde::Deserialize;
use std::fs::read_to_string;
pub struct Storage {
    db_client: mongodb::Database,
}
#[derive(Deserialize)]
struct UserCredentials {
    username: String,
    password: String,
}

impl Storage {
    pub async fn from_config() -> mongodb::error::Result<Storage> {
        let db_host = CONFIG
            .get_str("mongodb_host")
            .expect("Field mongodb_host not set in config.");
        let db_port = CONFIG
            .get_int("mongodb_port")
            .expect("Field mongodb_port not set in config.");
        Storage::new(db_host, db_port).await
    }

    pub async fn new(db_host: String, db_port: i64) -> mongodb::error::Result<Storage> {
        let uri = match get_credentials() {
            Ok(creds) => {
                info!("Trying to log in to database as {}", creds.username);
                format!(
                    "mongodb://{}:{}@{}:{}/dotastats?authSource=dotastats",
                    creds.username, creds.password, db_host, db_port
                )
            }
            Err(_) => {
                warn!("Unable to get credentials. Login in anonymously.");
                format!(
                    "mongodb://{}:{}/dotastats?authSource=dotastats",
                    db_host, db_port
                )
            }
        };
        let client = mongodb::Client::with_uri_str(uri.as_str()).await?;
        info!(
            "Succesfully connected with {}:{} database.",
            db_host, db_port
        );
        Ok(Storage {
            db_client: client.database("dotastats"),
        })
    }
}

fn get_credentials() -> Result<UserCredentials, BoxError> {
    match auto_authentication() {
        Ok(user_creds) => return Ok(user_creds),
        Err(e) => {
            error!(
                "Error ({}) occured during automatic authentication. Retrying manually.",
                e
            );
        }
    };
    manual_authentication()
}

fn auto_authentication() -> Result<UserCredentials, BoxError> {
    let creds_file = CONFIG.get_str("mongodb_user_file")?;
    let user_creds = read_to_string(creds_file)?;
    let user_creds: UserCredentials = serde_json::from_str(user_creds.as_str())?;
    Ok(user_creds)
}

fn manual_authentication() -> Result<UserCredentials, BoxError> {
    println!("Type user name: ");
    let mut username: String = String::new();
    std::io::stdin().read_line(&mut username)?;
    let password = scanpw::scanpw!("Type a password: ");
    Ok(UserCredentials { username, password })
}
