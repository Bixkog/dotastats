use crate::match_stats::Match;
use crate::analyzers::roles::{get_roles_wr, roles_wr_to_json, compress_roles_wr};
use crate::data_retrieval::retrieval_agent::process_guild_matches_retrieval;
use crate::storage::result_storage::store_roles_wr_result;


fn process_roles_wr(guild_id: &String, data: &Vec<Match>) -> Result<(), Box<dyn std::error::Error>> {
    let roles_wr = get_roles_wr(&data);
    let roles_wr = compress_roles_wr(roles_wr);
    let roles_wr_json = roles_wr_to_json(roles_wr);
    store_roles_wr_result(guild_id, roles_wr_json)?;
    Ok(())
}

pub async fn process_guild_request(guild_id: String, update: bool) -> Result<(), Box<dyn std::error::Error>> {
    let matches = process_guild_matches_retrieval(&guild_id, update).await?;
    process_roles_wr(&guild_id, &matches)?;
    Ok(())
}