use chrono::Timelike;
pub use serenity::{
    client::Context,
    model::interactions::{
        Interaction, InteractionApplicationCommandCallbackDataFlags, InteractionResponseType,
    },
    model::prelude::*,
};
pub use tracing::{error, info, warn};

use crate::context_ext::ContextExt;

pub async fn interaction_respond_with_private_message(
    ctx: Context,
    interaction: Interaction,
    content: &str,
) {
    match interaction {
        Interaction::ApplicationCommand(interaction) => interaction
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .content(content)
                            .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                    })
            })
            .await
            .unwrap_or_else(|why| warn!("Error responding to interaction: {}", why)),
        Interaction::MessageComponent(interaction) => interaction
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .content(content)
                            .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                    })
            })
            .await
            .unwrap_or_else(|why| warn!("Error responding to interaction: {}", why)),
        Interaction::Ping(_) => warn!("Cant respond to ping interaction!"),
    }
}

pub async fn update_bot_status(ctx: &Context) {
    if ctx.is_session_started().await {
        let game = ctx.session().await.read().await.game.name.clone();
        let content = format!("{} | Now!", game);
        ctx.set_presence(Some(Activity::playing(content)), OnlineStatus::DoNotDisturb)
            .await;
    } else if ctx.is_session_present().await {
        let game = ctx.session().await.read().await.game.name.clone();
        let time = ctx.session().await.read().await.time.time();
        let timezone = ctx.config().await.timezone_text;

        let hour = if time.hour() < 10 {
            format!("0{}", time.hour())
        } else {
            time.hour().to_string()
        };
        let minute = if time.minute() < 10 {
            format!("0{}", time.minute())
        } else {
            time.minute().to_string()
        };

        let content = format!("{} | {}:{} {}", game, hour, minute, timezone);
        ctx.set_presence(Some(Activity::playing(content)), OnlineStatus::Idle)
            .await;
    } else {
        let content = ctx.config().await.idle_text;
        ctx.set_presence(Some(Activity::playing(content)), OnlineStatus::Online)
            .await;
    }
}
