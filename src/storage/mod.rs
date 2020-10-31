pub mod guild_data_storage;
pub mod result_storage;
use crate::CONFIG;

pub struct Storage {
    db_client: mongodb::Client,
}

impl Storage {
    pub async fn from_config() -> mongodb::error::Result<Storage> {
        let db_host = CONFIG
            .get_str("mongodb_host")
            .expect("Field mongodb_host not set in config.");
        let db_port = CONFIG
            .get_int("mongodb_port")
            .expect("Field mongodb_port not set in config.");
        let db_uri = format!("mongodb://{}:{}/", db_host, db_port);
        Ok(Storage {
            db_client: mongodb::Client::with_uri_str(db_uri.as_str()).await?,
        })
    }
    pub async fn new(db_host: String, db_port: i64) -> mongodb::error::Result<Storage> {
        let db_uri = format!("mongodb://{}:{}/", db_host, db_port);
        Ok(Storage {
            db_client: mongodb::Client::with_uri_str(db_uri.as_str()).await?,
        })
    }
}
