use std::cmp::Ordering;
use serde::{Deserialize, Serialize};

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
