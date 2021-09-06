use std::sync::Arc;

use crate::{
    context_ext::ContextExt,
    interaction_handler::{CommandHandler, InteractionHandler, MessageHandler},
    session::{Session, UserState},
};

use super::{
    prelude::{interaction_respond_with_private_message, update_bot_status},
    status::get_status_embed,
};
use chrono::{DateTime, Local, NaiveDateTime, NaiveTime, TimeZone};
use serenity::{
    async_trait,
    client::Context,
    model::{
        channel::Message,
        id::{ChannelId, RoleId},
        interactions::{
            application_command::{
                ApplicationCommandInteraction, ApplicationCommandInteractionDataOptionValue,
                ApplicationCommandOptionType,
            },
            message_component::{ButtonStyle, MessageComponentInteraction},
            Interaction, InteractionResponseType,
        },
    },
    prelude::RwLock,
};
use tracing::warn;

#[derive(Clone, Copy)]
pub struct HostGame;

async fn ping_all_not_in_vc(ctx: &Context, channel_id: ChannelId) {
    let user_map = ctx.session().await.read().await.users.clone();
    let members = ctx
        .config()
        .await
        .vc_channel
        .to_channel(&ctx.http)
        .await
        .expect("Could not convert to Channel")
        .guild()
        .expect("Could not convert to GuildChannel")
        .members(&ctx.cache)
        .await
        .expect("Could not retrieve Member list");

    let pings = user_map
        .iter()
        .filter(|(u, s)| **s == UserState::Will && !members.iter().any(|m| m.user.id == **u))
        .fold(String::new(), |lhs, (rhs, _)| {
            lhs + format!("<@{}> ", rhs).as_str()
        });

    if pings.is_empty() {
        return;
    }

    let content = format!("{}you're late, get in the VC!", pings);
    if let Err(why) = channel_id
        .send_message(&ctx.http, |message| message.content(content))
        .await
    {
        warn!("Error sending message to text channel: {}", why);
    }
}

async fn start_session(
    ctx: &Context,
    interaction: &ApplicationCommandInteraction,
    time: &str,
    description: &str,
) -> bool {
    let channel_id = interaction.channel_id;
    let guild_id = interaction.guild_id.unwrap_or_default();

    let session_time =
        NaiveTime::parse_from_str(time, "%H:%M").expect("Error parsing default time to string");
    let now = Local::now();
    let today = Local::today();
    let session_time = Local
        .from_local_datetime(&NaiveDateTime::new(today.naive_local(), session_time))
        .earliest()
        .expect("Error parsing time to DateTime");

    let session_time = if (session_time - now) < chrono::Duration::zero() {
        session_time
            .date()
            .succ()
            .and_time(session_time.time())
            .unwrap()
    } else {
        session_time
    };

    let ctx2 = ctx.clone();
    let handle = tokio::task::spawn(async move {
        let ctx = ctx2.clone();
        let ten_minutes_before =
            session_time.signed_duration_since(now) - chrono::Duration::minutes(10);

        tokio::time::sleep(
            ten_minutes_before
                .to_std()
                .unwrap_or_else(|_| std::time::Duration::from_secs(60)),
        )
        .await;

        let game = ctx.session().await.read().await.game.clone();
        let embed = get_status_embed(&ctx, guild_id).await;

        channel_id
            .send_message(&ctx.http, |message| {
                message
                    .set_embed(embed)
                    .content(format!(
                        "<@&{}> Session starting soon!",
                        game.role_id.to_string()
                    ))
                    .allowed_mentions(|mentions| mentions.roles(vec![game.role_id]))
            })
            .await
            .expect("Error sending message to channel");

        tokio::time::sleep(
            session_time
                .signed_duration_since(now)
                .to_std()
                .unwrap_or_default(),
        )
        .await;

        let member_amount = ctx
            .session()
            .await
            .read()
            .await
            .users
            .iter()
            .filter(|(_, s)| **s == UserState::Will)
            .count();

        let embed = get_status_embed(&ctx, guild_id).await;
        let person_or_people = if member_amount == 1 {
            "person"
        } else {
            "people"
        };

        channel_id
            .send_message(&ctx.http, |message| {
                message.set_embed(embed).content(format!(
                    "{} Session has started! {} {} said Yes!",
                    game.name, member_amount, person_or_people
                ))
            })
            .await
            .expect("Error sending message to channel");

        update_bot_status(&ctx).await;

        tokio::time::sleep(std::time::Duration::from_secs(60 * 10)).await;
        // ping users who said yes but not in VC
        ping_all_not_in_vc(&ctx, channel_id).await;
    });

    let game = match ctx
        .config()
        .await
        .games
        .iter()
        .find(|g| g.channel_id == Some(channel_id))
    {
        Some(g) => g,
        None => {
            handle.abort();

            return false;
        }
    }
    .clone();

    let message = send_session_message(
        ctx.clone(),
        interaction,
        session_time,
        description,
        game.role_id,
    )
    .await;

    ctx.data
        .write()
        .await
        .insert::<Session>(Arc::new(RwLock::new(Session::new(
            game,
            handle,
            session_time,
            message.id,
            interaction.user.id,
        ))));
    update_bot_status(ctx).await;

    true
}

async fn send_session_message(
    ctx: Context,
    interaction: &ApplicationCommandInteraction,
    time: DateTime<Local>,
    description: &str,
    role_id: RoleId,
) -> Message {
    let description = if description.is_empty() {
        description.to_string()
    } else {
        format!("Description: {}", description)
    };

    interaction
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message
                        .content(format!(
                            "<@&{}> A session is planned!\nTime: <t:{}>\n{}",
                            role_id,
                            time.timestamp(),
                            description
                        ))
                        .allowed_mentions(|mentions| mentions.roles(vec![role_id]))
                        .components(|component| {
                            component.create_action_row(|row| {
                                row.create_button(|button| {
                                    button
                                        .custom_id("button-yes")
                                        .label("Yes")
                                        .style(ButtonStyle::Success)
                                })
                                .create_button(|button| {
                                    button
                                        .custom_id("button-maybe")
                                        .label("Maybe")
                                        .style(ButtonStyle::Secondary)
                                })
                                .create_button(|button| {
                                    button
                                        .custom_id("button-no")
                                        .label("No")
                                        .style(ButtonStyle::Danger)
                                })
                            })
                        })
                })
        })
        .await
        .expect("Error responding to interaction");

    let message = interaction
        .get_interaction_response(&ctx.http)
        .await
        .expect("Error retrieving interaction response");

    message.pin(&ctx.http).await.expect("Error pinning message");
    message
}

impl InteractionHandler for HostGame {
    fn name(&self) -> &'static str {
        "hostgame"
    }
}

#[async_trait]
impl CommandHandler for HostGame {
    async fn invoke(&self, ctx: Context, interaction: ApplicationCommandInteraction) {
        if ctx.is_session_present().await {
            interaction_respond_with_private_message(
                &ctx,
                &Interaction::ApplicationCommand(interaction),
                "There is already a session running!",
            )
            .await;
            return;
        }

        let config = ctx.config().await;
        let mut time = config.default_time;
        let mut description = String::new();

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

        if !start_session(&ctx, &interaction, &time, &description).await {
            interaction_respond_with_private_message(
                &ctx,
                &Interaction::ApplicationCommand(interaction),
                "This is not a game channel!",
            )
            .await;
        }
    }

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

#[derive(Clone, Copy)]
pub struct ButtonYes;

#[derive(Clone, Copy)]
pub struct ButtonMaybe;

#[derive(Clone, Copy)]
pub struct ButtonNo;

impl InteractionHandler for ButtonYes {
    fn name(&self) -> &'static str {
        "button-yes"
    }
}

#[async_trait]
impl MessageHandler for ButtonYes {
    async fn invoke(&self, ctx: Context, interaction: MessageComponentInteraction) {
        let user_id = interaction.user.id;
        interaction_respond_with_private_message(
            &ctx,
            &Interaction::MessageComponent(interaction),
            format!("Thanks for saying yes, <@{}>", user_id).as_str(),
        )
        .await;

        ctx.session()
            .await
            .write()
            .await
            .users
            .insert(user_id, crate::session::UserState::Will);
    }
}

#[async_trait]
impl InteractionHandler for ButtonMaybe {
    fn name(&self) -> &'static str {
        "button-maybe"
    }
}

#[async_trait]
impl MessageHandler for ButtonMaybe {
    async fn invoke(&self, ctx: Context, interaction: MessageComponentInteraction) {
        let user_id = interaction.user.id;
        interaction_respond_with_private_message(
            &ctx,
            &Interaction::MessageComponent(interaction),
            format!("Thanks for saying maybe, <@{}>", user_id).as_str(),
        )
        .await;

        ctx.session()
            .await
            .write()
            .await
            .users
            .insert(user_id, crate::session::UserState::May);
    }
}

impl InteractionHandler for ButtonNo {
    fn name(&self) -> &'static str {
        "button-no"
    }
}

#[async_trait]
impl MessageHandler for ButtonNo {
    async fn invoke(&self, ctx: Context, interaction: MessageComponentInteraction) {
        let user_id = interaction.user.id;
        interaction_respond_with_private_message(
            &ctx,
            &Interaction::MessageComponent(interaction),
            format!("Thanks for saying no, <@{}>", user_id).as_str(),
        )
        .await;

        ctx.session()
            .await
            .write()
            .await
            .users
            .insert(user_id, crate::session::UserState::Wont);
    }
}
