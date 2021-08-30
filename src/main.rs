mod commands;
mod config;
mod session;

use hotwatch::Hotwatch;
use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        interactions::{Interaction, InteractionType},
    },
    prelude::*,
};
use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::runtime::Handle;
use tracing::{error, info, warn};

use crate::commands::{
    buttons::ButtonYes,
    interaction_handler::{register_interaction_handler, InteractionMap},
};
use crate::commands::{
    buttons::{ButtonMaybe, ButtonNo},
    ping::Ping,
};
use crate::commands::{hostgame::HostGame, interaction_handler::register_guild_command};
use crate::config::Config;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let map = ctx
            .data
            .read()
            .await
            .get::<InteractionMap>()
            .expect("There was an error retrieving the InteractionMap")
            .read()
            .await
            .clone();

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

        register_guild_command(ctx.clone(), 699271154065735771_u64, Ping).await;
        register_guild_command(ctx.clone(), 699271154065735771_u64, HostGame).await;
        register_interaction_handler(ctx.clone(), ButtonYes).await;
        register_interaction_handler(ctx.clone(), ButtonMaybe).await;
        register_interaction_handler(ctx.clone(), ButtonNo).await;
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::read_from(Path::new("config.toml")).expect("Could not open config.toml");
    let token = config.discord_token.clone();
    let application_id: u64 = config.application_id;

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .application_id(application_id)
        .await
        .expect("Error creating client");
    let data = client.data.clone();

    data.write()
        .await
        .insert::<Config>(Arc::new(RwLock::new(config)));

    let handle = Handle::current();
    let mut hotwatch = Hotwatch::new().expect("Hotwatch failed to initialize!");
    hotwatch
        .watch("config.toml", move |event| {
            if let hotwatch::Event::Write(_) = event {
                if let Some(config) = Config::read_from(Path::new("config.toml")) {
                    info!("Config changed!");
                    for game in &config.games {
                        info!("{}", game.name);
                    }

                    handle.block_on(async {
                        data.write()
                            .await
                            .insert::<Config>(Arc::new(RwLock::new(config)));
                    });
                }
            }
        })
        .expect("Failed to watch config.toml");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
