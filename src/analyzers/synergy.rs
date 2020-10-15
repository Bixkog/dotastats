use crate::match_stats::{Match};
use std::collections::HashMap;
use super::analyzers_utils::*;

fn lane_name(lane: u64) -> String {
    match lane {
        0 => "Safe-lane".to_string(),
        1 => "Mid-lane".to_string(),
        2 => "Hard-lane".to_string(),
        l => format!("(Unknown lane: {})", l).to_string(),
    }
}

pub fn lane_synergy(data: &Vec<Match>) -> Vec<(String, (f64, [WinRatio; 3]))> {
    println!("Calculating synergy.");
    let mut lane_wr: HashMap<((String, String), u64), WinRatio> = HashMap::new();
    let mut solo_wr: HashMap<(String, u64), WinRatio> = HashMap::new();
    let mut processed_matches = 0;
    for match_ in data.iter() {
        let lane_up = skip_fail!(match_.get_team_laning());
        let is_won = skip_fail!(match_.is_won());
        processed_matches += 1;
        let mut lanes: [Vec<String>; 5] = Default::default();
        for (player, lane) in lane_up.iter() {
            let lane = *lane - 1;
            lanes[lane as usize].push(player.clone());
            solo_wr
                .entry((player.clone(), lane))
                .or_default()
                .add_score(is_won);
        }
        for (lane, players) in lanes.iter_mut().enumerate() {
            if players.len() != 2 {
                continue;
            }
            players.sort();
            let players = (players[0].clone(), players[1].clone());
            lane_wr
                .entry((players, lane as u64))
                .or_default()
                .add_score(is_won);
        }
    }
    println!("Processed {} matches.", processed_matches);
    let mut synergies: Vec<(((String, String), u64), (f64, [WinRatio; 3]))> = Vec::new();
    for ((p, lane), wr) in lane_wr {
        let p1_solo_wr = &solo_wr[&(p.0.clone(), lane)];
        let p2_solo_wr = &solo_wr[&(p.1.clone(), lane)];
        let synergy = wr.as_percent() / ((p1_solo_wr.as_percent() + p2_solo_wr.as_percent()) / 2.0);
        synergies.push(((p, lane), (synergy, [wr.clone(), p1_solo_wr.clone(), p2_solo_wr.clone()])));
    }
    println!("Calculated {} synergies.", synergies.len());
    synergies.sort_by(|(_, v1), (_, v2)| v2.partial_cmp(v1).unwrap());
    synergies.reverse();
    synergies.iter().map(|(k, v)| ([k.0.0.clone(), k.0.1.clone(), lane_name(k.1)].join(" "), v.clone())).collect()
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn synergy_test() {
        let mock_matches : Vec<Match> = vec![
            serde_json::from_str(r#"{"match_stats": {}, "players_stats": [{"personaname" : "test1", "lane" : 1, "win" : 1}, 
                                                    {"personaname" : "test2", "lane" : 1, "win" : 1}]}"#).unwrap(),
            serde_json::from_str(r#"{"match_stats": {}, "players_stats": [{"personaname" : "test1", "lane" : 1, "win" : 1}]}"#).unwrap(),
            serde_json::from_str(r#"{"match_stats": {}, "players_stats": [{"personaname" : "test2", "lane" : 1, "win" : 1}]}"#).unwrap(),
            serde_json::from_str(r#"{"match_stats": {}, "players_stats": [{"personaname" : "test1", "lane" : 1, "win" : 0}]}"#).unwrap(),
            serde_json::from_str(r#"{"match_stats": {}, "players_stats": [{"personaname" : "test2", "lane" : 1, "win" : 0}]}"#).unwrap(),
        ];

        let expected_result: Vec<(String, (f64, [WinRatio; 3]))> = vec![
            ("test1 test2 Safe-lane".to_string(), (1.5, [WinRatio{wins: 1, looses: 0}, WinRatio{wins: 2, looses: 1}, WinRatio{wins: 2, looses: 1}])),
        ];
        assert_eq!(expected_result, lane_synergy(&mock_matches));
    }
}
