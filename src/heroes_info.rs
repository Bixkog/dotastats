use serde_json;
use std::collections::HashMap;
use std::fs::read_to_string;

use crate::BoxError;
use serde::de::Error;
use serde_json::error::Error as serde_error;

/// Hero data relevant for analysis.
#[derive(Clone, Default)]
pub struct Hero {
    pub name: String,
    pub roles: Vec<String>,
}

/// Contains map HeroId -> Hero
pub struct HeroesInfo {
    heroes: HashMap<u64, Hero>,
}

impl HeroesInfo {
    /// Initializes from json file with heroes constants.
    /// File may be retrieved by
    /// > wget https://raw.githubusercontent.com/odota/dotaconstants/master/build/heroes.json
    pub fn init(heroes_filename: String) -> Result<Self, BoxError> {
        let raw_heroes: serde_json::Value =
            serde_json::from_str(read_to_string(heroes_filename)?.as_str())?;
        let heroes_constants = raw_heroes
            .as_object()
            .ok_or(serde_error::custom("Heroes constants is not json object."))?;
        let mut heroes_info = HeroesInfo {
            heroes: HashMap::new(),
        };
        for (id, hero) in heroes_constants {
            let hero_id = id.parse::<u64>()?;
            assert!(hero_id > 0);
            let hero_parsed = Hero {
                name: hero["localized_name"]
                    .as_str()
                    .ok_or(serde_error::custom(format!(
                        "no field localized_name for hero_id: {}",
                        hero_id
                    )))?
                    .to_string(),
                roles: hero["roles"]
                    .as_array()
                    .ok_or(serde_error::custom(format!(
                        "no field roles for hero_id: {}",
                        hero_id
                    )))?
                    .iter()
                    .map(|v| {
                        Ok(v.as_str()
                            .ok_or(serde_error::custom(format!(
                                "roles is not string for hero_id: {}",
                                hero_id
                            )))?
                            .to_string())
                    })
                    .collect::<Result<Vec<String>, serde_json::Error>>()?,
            };
            heroes_info.heroes.insert(hero_id, hero_parsed);
        }
        Ok(heroes_info)
    }

    pub fn get_hero(&self, hero_id: u64) -> Hero {
        self.heroes[&hero_id].clone()
    }
}
