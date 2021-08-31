use std::{fs::File, io::Read, path::Path, sync::Arc};

use serde::Deserialize;
use serenity::prelude::{RwLock, TypeMapKey};
use tracing::log::error;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub application_id: u64,
    pub discord_token: String,
    pub default_time: String,
    pub default_description: String,
    pub ip_message: String,
    pub games: Vec<Game>,
}

#[derive(Deserialize, Clone)]
pub struct Game {
    pub name: String,
    pub channel_id: u64,
    pub role_id: u64,
}

impl TypeMapKey for Config {
    type Value = Arc<RwLock<Config>>;
}

impl Config {
    pub fn read_from(path: &Path) -> Option<Self> {
        let mut config_file = match File::open(path) {
            Ok(f) => f,
            Err(why) => {
                error!("Error opening {:?}: {}", path, why);
                return None;
            }
        };

        let mut config_str = String::new();
        if let Err(why) = config_file.read_to_string(&mut config_str) {
            error!("Error reading {:?}: {}", path, why);
            return None;
        }

        match toml::from_str(&config_str) {
            Ok(config) => Some(config),
            Err(why) => {
                error!("Error parsing {:?} to config: {}", path, why);
                None
            }
        }
    }
}
