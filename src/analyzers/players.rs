use crate::analyzers::WinRatio;
use crate::match_stats::Match;
use crate::match_stats::PlayerName;
use itertools::Itertools;
use std::collections::HashMap;

/// Computes winratio for each Player setup.
pub fn get_players_wr(matches: &Vec<Match>) -> Vec<(Vec<PlayerName>, WinRatio)> {
    let mut players_score: HashMap<Vec<PlayerName>, WinRatio> = HashMap::new();
    for match_ in matches {
        let mut team = skip_fail!(match_.get_team());
        team.sort();
        let is_won = skip_fail!(match_.is_won());
        for i in 1..=(team.len()) {
            for subset in team.clone().into_iter().combinations(i) {
                players_score.entry(subset).or_default().add_score(is_won);
            }
        }
    }
    players_score.into_iter().collect()
}
