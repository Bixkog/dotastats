use crate::data_retrieval::parse_requester::ParseRequester;
use crate::data_retrieval::data_retriever::DataRetriever;
use crate::match_stats::Match;
use crate::data_retrieval::match_updater::MatchUpdater;
use crate::data_retrieval::extractor::extract_stats;

pub async fn process_guild_matches_retrieval(guild_id: &String, update: bool) -> reqwest::Result<Vec<Match>> {
    if update {
        let match_updater = MatchUpdater::new();
        match_updater.try_update(&guild_id).await?;
    }
    let data_retriever = DataRetriever::new();
    let guild_raw_data = data_retriever.get_guild_raw_data(&guild_id).await?;
    let parse_requester = ParseRequester::new();
    parse_requester
        .request_parsing(&guild_raw_data.guild_id, &guild_raw_data.members_matches)
        .await?;
    let matches = extract_stats(guild_raw_data).unwrap();
    Ok(matches)
}