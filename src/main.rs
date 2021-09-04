mod commands;
mod config;
mod context_ext;
mod embed;
mod session;

use hotwatch::Hotwatch;
use serenity::{
    async_trait,
    model::{gateway::Ready, interactions::Interaction},
    prelude::*,
};
use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::runtime::Handle;
use tracing::{error, info, warn};

use crate::{
    commands::{
        endhost::{self, EndHost},
        help::{Help, HelpPageHandler},
        hostgame::{self, HostGame},
        interaction_handler::{register_guild_command, register_handler, Handler, InteractionMap},
        ip::Ip,
        prelude::*,
        status::Status,
    },
    config::Config,
    context_ext::ContextExt,
};

struct ClientHandler;

#[async_trait]
impl EventHandler for ClientHandler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let map = ctx.interaction_map().await;

        match interaction {
            Interaction::ApplicationCommand(interaction) => {
                let name = interaction.data.name.clone();
                if let Some(Handler::Command(command)) = map.get(name.as_str()) {
                    command.invoke(ctx.clone(), interaction).await;
                } else {
                    warn!("Slash command not found in map: {}", name);
                }
            }
            Interaction::MessageComponent(interaction) => {
                let name = interaction.data.custom_id.clone();
                if let Some(Handler::Message(message_handler)) = map.get(name.as_str()) {
                    message_handler.invoke(ctx.clone(), interaction).await;
                } else {
                    warn!("Message handler not found in map: {}", name);
                }
            }
            Interaction::Ping(_) => {
                error!("Error: interaction kind not supported: {:?}", interaction)
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connected as user: {}", ready.user.name);

        let map = HashMap::new();
        ctx.data
            .write()
            .await
            .insert::<InteractionMap>(Arc::new(RwLock::new(map)));

        let guild_id = ctx.config().await.guild_id;

        register_guild_command(&ctx, guild_id, EndHost).await;
        register_guild_command(&ctx, guild_id, Help).await;
        register_guild_command(&ctx, guild_id, HostGame).await;
        register_guild_command(&ctx, guild_id, Ip).await;
        register_guild_command(&ctx, guild_id, Status).await;

        register_handler(&ctx, Handler::Message(Arc::new(hostgame::ButtonYes))).await;
        register_handler(&ctx, Handler::Message(Arc::new(hostgame::ButtonMaybe))).await;
        register_handler(&ctx, Handler::Message(Arc::new(hostgame::ButtonNo))).await;
        register_handler(&ctx, Handler::Message(Arc::new(endhost::ButtonYes))).await;
        register_handler(&ctx, Handler::Message(Arc::new(endhost::ButtonNo))).await;
        register_handler(&ctx, Handler::Message(Arc::new(HelpPageHandler))).await;

        update_bot_status(&ctx).await;
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::read_from(Path::new("config.toml")).expect("Could not open config.toml");
    let token = config.discord_token.clone();

    let mut client = Client::builder(token)
        .event_handler(ClientHandler)
        .application_id(config.application_id.0)
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
