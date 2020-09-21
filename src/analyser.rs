use crate::match_stats::{Match, PlayerSetup};
use std::collections::HashMap;
use itertools::Itertools;

macro_rules! skip_fail {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(_) => continue,
        }
    };
}

pub type WinRatio = (/*wins:*/ u32, /*looses:*/ u32);

fn prehash_team_setup(mut team_setup : Vec<&PlayerSetup>) -> String {
    //team_setup.sort_by(|a, b|   a.0.cmp(&b.0));
    //team_setup.iter().map(|ps| format!("{}_{}", ps.0, ps.1)).collect::<Vec<String>>().join(" ")
    team_setup.sort();
    team_setup.iter().map(|s| s.clone()).join(" ")
}

pub fn best_team_setup(data : &Vec<Match>) -> Vec<(String, WinRatio)> {
    let mut setup_score : HashMap<String, WinRatio> = HashMap::new();
    let mut counter = 0;
    for match_ in data.iter() {
        let team_setup = skip_fail!(match_.get_team_setup());
        let is_won = skip_fail!(match_.is_won());
        counter += 1;
        for i in 3..team_setup.len()+1 {
            for subteam_setup in team_setup.iter().combinations(i) {
                let entry = setup_score.entry(prehash_team_setup(subteam_setup)).or_insert((0, 0));
                if is_won {
                    entry.0 += 1;
                } 
                else {
                    entry.1 += 1;
                }
            }
        }
        
    }
    println!("Processed {} matches.", counter);
    let mut res = setup_score.into_iter().collect::<Vec<(String, WinRatio)>>();
    res.sort_by(|(_, wr_a), (_, wr_b)| (wr_a.1 * wr_b.0).cmp(&(wr_a.0 * wr_b.1)));
    println!("Found score for {} teams.", res.len());
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn best_team_setup_test() {
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

        let expected_result : Vec<(String, WinRatio)> = vec![("test1_1 test2_1".to_string(), (3, 1)), 
                                                            ("test2_1 test3_1".to_string(), (2, 1)), 
                                                            ("test1_1 test3_1".to_string(), (1, 1)), 
                                                            ("test1_1 test2_1 test3_1".to_string(), (0, 1))];
        assert_eq!(expected_result, best_team_setup(&mock_matches));
    }
}
