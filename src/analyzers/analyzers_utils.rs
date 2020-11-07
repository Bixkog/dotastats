use crate::heroes_info::Hero;
use crate::heroes_info::HeroesInfo;
use crate::match_stats::Match;
use crate::match_stats::PlayerName;

#[macro_export]
macro_rules! skip_fail {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(_) => continue,
        }
    };
}

/// Finds heroes played by players.
pub fn get_heroes(
    heroes_info: &HeroesInfo,
    match_: &Match,
    mut team: Vec<String>,
) -> Vec<(PlayerName, Hero)> {
    let mut team_setup = vec![];
    team.sort();
    for player in team {
        let player_hero_id = skip_fail!(match_.get_player_hero(&player));
        let hero = heroes_info.get_hero(player_hero_id);
        team_setup.push((player, hero));
    }
    team_setup
}
