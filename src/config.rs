use std::{fs::File, io::Read, path::Path, sync::Arc};

use serde::Deserialize;
use serenity::{
    async_trait,
    client::Context,
    prelude::{RwLock, TypeMapKey},
};
use tracing::log::error;

use crate::session::Session;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub application_id: u64,
    pub discord_token: String,
    pub default_time: String,
    pub default_description: String,
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

#[async_trait]
pub trait ContextExt {
    async fn config(&self) -> Config;
    async fn session(&self) -> Arc<RwLock<Session>>;
}

#[async_trait]
impl ContextExt for Context {
    async fn config(&self) -> Config {
        self.data
            .read()
            .await
            .get::<Config>()
            .expect("Error reading config from TypeMap")
            .read()
            .await
            .clone()
    }

    async fn session(&self) -> Arc<RwLock<Session>> {
        self.data
            .read()
            .await
            .get::<Session>()
            .expect("Error reading session from TypeMap")
            .clone()
    }
}
