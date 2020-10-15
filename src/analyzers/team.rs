use super::analyzers_utils::*;
use crate::match_stats::{Match, PlayerName};
use itertools::Itertools;
use std::collections::HashMap;

fn prehash_team(mut team_setup: Vec<&PlayerName>) -> String {
    team_setup.sort();
    team_setup.iter().map(|s| s.clone()).join(" ")
}

pub fn best_teams(data: &Vec<Match>) -> Vec<(String, WinRatio)> {
    println!("Finding best teams.");
    let mut team_score: HashMap<String, WinRatio> = HashMap::new();
    let mut processed_matches = 0;
    for match_ in data.iter() {
        let team = skip_fail!(match_.get_team());
        let is_won = skip_fail!(match_.is_won());
        processed_matches += 1;
        for i in 2..team.len() + 1 {
            for subteam in team.iter().combinations(i) {
                team_score
                    .entry(prehash_team(subteam))
                    .or_default()
                    .add_score(is_won);
            }
        }
    }
    println!("Processed {} matches.", processed_matches);
    let mut res = team_score.into_iter().collect::<Vec<(String, WinRatio)>>();
    res.sort_by_key(|(_, wr)| wr.clone());
    res.reverse();
    println!("Found score for {} teams.", res.len());
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn best_team_test() {
        let mock_matches : Vec<Match> = vec![
            serde_json::from_str(r#"{"match_stats": {}, "players_stats": [{"personaname" : "test1", "lane" : 1, "win" : 1}, 
                                                    {"personaname" : "test2", "lane" : 1, "win" : 1}]}"#).unwrap(),
            serde_json::from_str(r#"{"match_stats": {}, "players_stats": [{"personaname" : "test1", "lane" : 1, "win" : 1}, 
                                                    {"personaname" : "test2", "lane" : 1, "win" : 1}]}"#).unwrap(),
            serde_json::from_str(r#"{"match_stats": {}, "players_stats": [{"personaname" : "test1", "lane" : 1, "win" : 1}, 
                                                    {"personaname" : "test2", "lane" : 1, "win" : 1}]}"#).unwrap(),
            serde_json::from_str(r#"{"match_stats": {}, "players_stats": [{"personaname" : "test3", "lane" : 1, "win" : 1}, 
                                                    {"personaname" : "test2", "lane" : 1, "win" : 1}]}"#).unwrap(),
            serde_json::from_str(r#"{"match_stats": {}, "players_stats": [{"personaname" : "test3", "lane" : 1, "win" : 1}, 
                                                    {"personaname" : "test2", "lane" : 1, "win" : 1}]}"#).unwrap(),
            serde_json::from_str(r#"{"match_stats": {}, "players_stats": [{"personaname" : "test3", "lane" : 1, "win" : 1}, 
                                                    {"personaname" : "test1", "lane" : 1, "win" : 1}]}"#).unwrap(),
            serde_json::from_str(r#"{"match_stats": {}, "players_stats": [{"personaname" : "test1", "lane" : 1, "win" : 0}, 
                                                    {"personaname" : "test2", "lane" : 1, "win" : 0}, 
                                                    {"personaname" : "test3", "lane" : 1, "win" : 0}]}"#).unwrap(),
        ];

        let expected_result: Vec<(String, WinRatio)> = vec![
            ("test1 test2".to_string(), WinRatio { wins: 3, looses: 1 }),
            ("test2 test3".to_string(), WinRatio { wins: 2, looses: 1 }),
            ("test1 test3".to_string(), WinRatio { wins: 1, looses: 1 }),
            (
                "test1 test2 test3".to_string(),
                WinRatio { wins: 0, looses: 1 },
            ),
        ];
        assert_eq!(expected_result, best_teams(&mock_matches));
    }
}
