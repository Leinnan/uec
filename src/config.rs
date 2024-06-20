use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Config {
    pub editor_path: String,
}

impl Config {
    pub fn load_or_create() -> Self {
        confy::load("uec", "config").unwrap_or_default()
    }

    pub fn save(&self) {
        let _ = confy::store("uec", "config", self);
    }
}
