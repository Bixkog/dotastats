use crate::BoxError;
use std::sync::Arc;

use crate::data_retrieval::extractor::extract_stats;
use crate::match_stats::Match;
use crate::{data_retrieval::data_retriever::DataRetriever, storage::Storage};
use lazy_static;
use tokio::sync::Mutex;

lazy_static! {
    static ref RETRIEVAL_MUTEX: Mutex<()> = Mutex::new(());
}

pub async fn process_guild_matches_retrieval(
    storage: Arc<Storage>,
    guild_id: &String,
) -> Result<Vec<Match>, BoxError> {
    let _lock = RETRIEVAL_MUTEX.lock().await;
    let data_retriever = DataRetriever::new(storage);
    let guild_raw_data = data_retriever.get_guild_raw_data(&guild_id).await?;
    let matches = extract_stats(guild_raw_data)?;
    Ok(matches)
}
