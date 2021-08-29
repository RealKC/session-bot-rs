mod commands;

use serde::Deserialize;
use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        interactions::{Interaction, InteractionType},
    },
    prelude::*,
};
use std::{collections::HashMap, fs::File, io::Read, sync::Arc};
use tracing::{error, info, warn};

use crate::commands::interaction_handler::{register_guild_interaction_handler, InteractionMap};
use crate::commands::ping::Ping;

#[derive(Deserialize)]
struct Config {
    application_id: u64,
    discord_token: String,
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let data = ctx.data.read().await;
        let map = data
            .get::<InteractionMap>()
            .expect("There was an error retrieving the InteractionMap")
            .write()
            .await;

        match interaction.kind() {
            InteractionType::ApplicationCommand => {
                // UNWRAP SAFETY: interaction is always a ApplicationCommand
                let name = interaction.clone().application_command().unwrap().data.name;
                if let Some(handler) = map.get(name.as_str()) {
                    handler.invoke(ctx.clone(), interaction).await;
                } else {
                    warn!("Slash command not found in map: {}", name);
                }
            }
            InteractionType::MessageComponent => {
                // UNWRAP SAFETY: interaction is always a MessageComponent
                let name = interaction
                    .clone()
                    .message_component()
                    .unwrap()
                    .data
                    .custom_id;
                if let Some(handler) = map.get(name.as_str()) {
                    handler.invoke(ctx.clone(), interaction).await;
                } else {
                    warn!("Slash command not found in map: {}", name);
                }
            }
            _ => error!("Error: interaction kind not recognized: {:?}", interaction),
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connected as user: {}", ready.user.name);

        let map = HashMap::new();
        ctx.data
            .write()
            .await
            .insert::<InteractionMap>(Arc::new(RwLock::new(map)));

        register_guild_interaction_handler(ctx.clone(), 699271154065735771_u64, Ping).await;
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let mut config_file = File::open("config.toml").expect("Error opening config.toml");
    let mut config_str = String::new();
    config_file
        .read_to_string(&mut config_str)
        .expect("Error reading config.toml");
    let config: Config = toml::from_str(&config_str).expect("Erorr parsing config.toml");

    let token = config.discord_token;
    let application_id: u64 = config.application_id;

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .application_id(application_id)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
