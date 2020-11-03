use serde::{Deserialize, Serialize};
use std::option::NoneError;

/// Struct representing players stats at some match.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerStats {
    #[serde(default)]
    personaname: Option<String>,
    #[serde(default)]
    hero_id: Option<u64>,
    #[serde(default)]
    win: Option<u64>,
    // kda stats
    #[serde(default)]
    kda: Option<f64>,
    #[serde(default)]
    kills: Option<u64>,
    #[serde(default)]
    deaths: Option<u64>,
    #[serde(default)]
    assists: Option<u64>,
    // exp/gold stats
    #[serde(default)]
    gold_per_min: Option<u64>,
    #[serde(default)]
    xp_per_min: Option<u64>,
    #[serde(default)]
    level: Option<u64>,
    // laning stats
    #[serde(default)]
    lane_kills: Option<u64>,
    #[serde(default)]
    lane: Option<u64>,
    #[serde(default)]
    lane_role: Option<u64>,
    #[serde(default)]
    is_roaming: Option<bool>,
    #[serde(default)]
    last_hits: Option<u64>,
    #[serde(default)]
    denies: Option<u64>,
    #[serde(default)]
    neutral_kills: Option<u64>,
    #[serde(default)]
    lane_efficiency: Option<f64>,
    //fighting stats
    #[serde(default)]
    hero_damage: Option<u64>,
    #[serde(default)]
    hero_healing: Option<u64>,
    #[serde(default)]
    stuns: Option<f64>, // Total stun duration of all stuns by the player
    //support stats
    #[serde(default)]
    camps_stacked: Option<u64>,
    #[serde(default)]
    creeps_stacked: Option<u64>,
    #[serde(default)]
    obs_placed: Option<u64>,
    #[serde(default)]
    sen_placed: Option<u64>,
    #[serde(default)]
    observer_kills: Option<u64>,
    #[serde(default)]
    sen_kills: Option<u64>,
    // pushing stats
    #[serde(default)]
    tower_damage: Option<u64>,
    #[serde(default)]
    tower_kills: Option<u64>,
    //misc stats
    #[serde(default)]
    purchase_tpscroll: Option<u64>,
    #[serde(default)]
    buyback_count: Option<u64>,
    #[serde(default)]
    courier_kills: Option<u64>,
    #[serde(default)]
    rune_pickups: Option<u64>,
    #[serde(default)]
    roshan_kills: Option<u64>,
}

/// Struct containing match global stats.
#[derive(Serialize, Deserialize, Debug)]
pub struct MatchStats {
    #[serde(default)]
    skill: Option<u64>, // Normal, High, Very High
}

/// Struct containing all data about some match.
#[derive(Serialize, Deserialize, Debug)]
pub struct Match {
    match_stats: MatchStats,
    players_stats: Vec<PlayerStats>,
}

pub type PlayerName = String;
pub type PlayerLane = (/*personaname:*/ String, /*lane:*/ u64);

#[derive(Debug)]
pub enum StatsError {
    MissingField(NoneError),
    NoTargetPlayer(),
}

impl From<NoneError> for StatsError {
    fn from(none: NoneError) -> StatsError {
        StatsError::MissingField(none)
    }
}

pub type StatsResult<T> = std::result::Result<T, StatsError>;

impl Match {
    pub fn new(match_stats: MatchStats, players_stats: Vec<PlayerStats>) -> Match {
        Match {
            match_stats,
            players_stats,
        }
    }

    pub fn is_won(&self) -> StatsResult<bool> {
        Ok(self.players_stats[0].win? == 1)
    }

    pub fn get_team(&self) -> StatsResult<Vec<PlayerName>> {
        let mut team = vec![];
        for p in self.players_stats.iter() {
            team.push(p.personaname.clone()?); //((p.personaname.clone()?, p.lane?));
        }
        Ok(team)
    }

    pub fn get_player_hero(&self, player_name: &PlayerName) -> StatsResult<u64> {
        for p in self.players_stats.iter() {
            if p.personaname.as_ref()? == player_name {
                return Ok(p.hero_id?);
            }
        }
        Err(StatsError::NoTargetPlayer())
    }

    pub fn get_team_laning(&self) -> StatsResult<Vec<PlayerLane>> {
        let mut team = vec![];
        for p in self.players_stats.iter() {
            team.push((p.personaname.clone()?, p.lane?));
        }
        Ok(team)
    }

    pub fn get_team_size(&self) -> usize {
        self.players_stats.len()
    }
}
