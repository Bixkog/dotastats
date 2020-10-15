use serde_json;
use std::collections::HashMap;
use std::fs::read_to_string;

#[derive(Clone, Default)]
pub struct Hero {
    pub name: String,
    pub roles: Vec<String>,
}

pub struct Constants {
    heroes: HashMap<u64, Hero>,
}

impl Constants {
    pub fn init(heroes_filename: String) -> Self {
        let raw_heroes = serde_json::from_str::<serde_json::Value>(
            read_to_string(heroes_filename)
                .expect("Can't open heroes contants file.")
                .as_str(),
        )
        .unwrap();
        let heroes_constants = raw_heroes
            .as_object()
            .expect("Heroes constants are not object.");
        let mut constants = Constants {
            heroes: HashMap::new(),
        };
        for (id, hero) in heroes_constants {
            let id_int = id.parse::<u64>().unwrap();
            assert!(id_int > 0, "hero id is 0.");
            let hero_parsed = Hero {
                name: hero["localized_name"].as_str().unwrap().to_string(),
                roles: hero["roles"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_string())
                    .collect(),
            };
            constants.heroes.insert(id_int, hero_parsed);
        }
        constants
    }

    pub fn get_hero(&self, hero_id: u64) -> Hero {
        self.heroes[&hero_id].clone()
    }
}
