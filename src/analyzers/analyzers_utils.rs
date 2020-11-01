use crate::heroes_info::Hero;
use crate::heroes_info::HeroesInfo;
use crate::match_stats::Match;
use crate::match_stats::PlayerName;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::ops::Add;
use std::ops::Sub;

#[macro_export]
macro_rules! skip_fail {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(_) => continue,
        }
    };
}

#[derive(Debug, Default, Eq, Clone, Serialize, Deserialize)]
pub struct WinRatio {
    pub wins: u32,
    pub looses: u32,
}

impl Add for WinRatio {
    type Output = WinRatio;
    fn add(self, other: WinRatio) -> <Self as std::ops::Add<WinRatio>>::Output {
        WinRatio {
            wins: self.wins + other.wins,
            looses: self.looses + other.looses,
        }
    }
}

impl Sub for WinRatio {
    type Output = Option<WinRatio>;
    fn sub(self, other: WinRatio) -> <Self as std::ops::Sub<WinRatio>>::Output {
        if self.wins < other.wins || self.looses < other.looses {
            return None;
        }
        Some(WinRatio {
            wins: self.wins - other.wins,
            looses: self.looses - other.looses,
        })
    }
}

impl WinRatio {
    pub fn add_score(&mut self, win: bool) {
        if win {
            self.wins += 1;
        } else {
            self.looses += 1;
        }
    }

    pub fn as_percent(&self) -> f64 {
        if self.looses == 0 {
            1.0
        } else {
            self.wins as f64 / (self.wins + self.looses) as f64
        }
    }

    pub fn total(&self) -> u32 {
        self.wins + self.looses
    }
}

impl Ord for WinRatio {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for WinRatio {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (other.looses * self.wins).cmp(&(other.wins * self.looses)) {
            Ordering::Equal => Some(self.total().cmp(&other.total())),
            ord => Some(ord),
        }
    }
}

impl PartialEq for WinRatio {
    fn eq(&self, other: &Self) -> bool {
        (self.looses * other.wins) == (self.wins * other.looses)
    }
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
