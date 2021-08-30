use std::sync::Arc;

use crate::{config::ContextExt, session::Session};

use super::interaction_handler::{Command, InteractionHandler};
use chrono::{Local, NaiveDateTime, NaiveTime, TimeZone};
use serenity::{
    async_trait,
    client::Context,
    model::{
        id::{ChannelId, RoleId},
        interactions::{
            application_command::{
                ApplicationCommandInteractionDataOptionValue, ApplicationCommandOptionType,
            },
            message_component::ButtonStyle,
            Interaction, InteractionResponseType,
        },
    },
    prelude::RwLock,
};
use tracing::{info, warn};

#[derive(Clone, Copy)]
pub struct HostGame;

async fn start_session(ctx: Context, time: &str, channel_id: u64) -> bool {
    let session_time =
        NaiveTime::parse_from_str(&time, "%H:%M").expect("Error parsing default time to string");
    let now = Local::now();
    let today = Local::today();
    let session_time = Local
        .from_local_datetime(&NaiveDateTime::new(today.naive_local(), session_time))
        .earliest()
        .expect("Error parsing time to DateTime");

    let (session_time, session_is_tomorrow) = if (session_time - now) < chrono::Duration::zero() {
        (
            session_time
                .date()
                .succ()
                .and_time(session_time.time())
                .unwrap(),
            true,
        )
    } else {
        (session_time, false)
    };

    let ctx2 = ctx.clone();
    let handle = tokio::task::spawn(async move {
        let ctx = ctx2.clone();
        let ten_minutes_before =
            session_time.signed_duration_since(now) - chrono::Duration::minutes(10);
        let ten_minutes_before = ten_minutes_before.to_std();
        tokio::time::sleep(ten_minutes_before.unwrap()).await;
        let game = ctx.session().await.read().await.game.clone();
        ChannelId(game.channel_id)
            .send_message(&ctx.http, |message| {
                message.content(format!("<@&{}>", RoleId(game.role_id).to_string()))
            })
            .await
            .expect("Error sending message to channel");

        // ping the role
        // tokio::time::sleep(std::time::Duration::from_secs(60 * 10 * 2));
        // ping users who said yes but not in VC
    });

    let game = match ctx
        .config()
        .await
        .games
        .iter()
        .find(|g| g.channel_id == channel_id)
    {
        Some(g) => g,
        None => {
            handle.abort();

            return false;
        }
    }
    .clone();

    ctx.data
        .write()
        .await
        .insert::<Session>(Arc::new(RwLock::new(Session::new(
            game,
            handle,
            session_is_tomorrow,
        ))));
    true
}

#[async_trait]
impl InteractionHandler for HostGame {
    fn name(&self) -> &'static str {
        "hostgame"
    }

    async fn invoke(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(interaction) = interaction {
            info!("{:#?}", interaction);

            let config = ctx.config().await;
            let mut time = config.default_time;
            let mut description = config.default_description;

            for option in &interaction.data.options {
                match option.name.as_ref() {
                    "time" => {
                        if let ApplicationCommandInteractionDataOptionValue::String(s) =
                            option.resolved.as_ref().unwrap()
                        {
                            time = s.clone();
                        }
                    }
                    "description" => {
                        if let ApplicationCommandInteractionDataOptionValue::String(s) =
                            option.resolved.as_ref().unwrap()
                        {
                            description = s.clone();
                        }
                    }
                    _ => {}
                }
            }

            let worked = start_session(ctx.clone(), &time, *interaction.channel_id.as_u64()).await;
            if worked {
                if let Err(why) = interaction
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| {
                                message
                                    .content(format!(
                                        "time: {}\ndescription: {}",
                                        time, description
                                    ))
                                    .components(|component| {
                                        component.create_action_row(|row| {
                                            row.create_button(|button| {
                                                button
                                                    .custom_id("button-yes")
                                                    .label("YES")
                                                    .style(ButtonStyle::Primary)
                                            })
                                            .create_button(|button| {
                                                button
                                                    .custom_id("button-maybe")
                                                    .label("MAYBE")
                                                    .style(ButtonStyle::Primary)
                                            })
                                            .create_button(|button| {
                                                button
                                                    .custom_id("button-no")
                                                    .label("NO")
                                                    .style(ButtonStyle::Primary)
                                            })
                                        })
                                    })
                            })
                    })
                    .await
                {
                    warn!("Error responding to slash command: {}", why);
                }
            } else {
                if let Err(why) = interaction
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| {
                                message.content(
                                    "Error creating session: No game registered to this channel",
                                )
                            })
                    })
                    .await
                {
                    warn!("Error responding to slash command: {}", why);
                }
            }
        }
    }
}

impl Command for HostGame {
    fn create_command(
        self,
        command: &mut serenity::builder::CreateApplicationCommand,
    ) -> &mut serenity::builder::CreateApplicationCommand {
        command
            .name(self.name())
            .description("Hosts a new game")
            .create_option(|option| {
                option
                    .kind(ApplicationCommandOptionType::String)
                    .name("time")
                    .description("Time to host the session")
            })
            .create_option(|option| {
                option
                    .kind(ApplicationCommandOptionType::String)
                    .name("description")
                    .description("Sets the session description")
            })
    }
}
