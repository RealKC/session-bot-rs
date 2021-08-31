use crate::context_ext::ContextExt;

use super::interaction_handler::{InteractionHandler, MessageHandler};
use serenity::{
    async_trait, client::Context,
    model::interactions::message_component::MessageComponentInteraction,
};
use tracing::log::warn;

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
        if let Err(why) = interaction
                .create_interaction_response(&ctx.http, |response| {
                    response.kind(serenity::model::interactions::InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.content(format!("thanks for saying yes, {}", interaction.user)))
                })
                .await
            {
                warn!("Error handling invocation: {}", why);
            } else {
                ctx.session().await.write().await.users.insert(interaction.user.id, crate::session::UserState::WillJoin);
        }
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
        if let Err(why) = interaction
            .create_interaction_response(&ctx.http, |response| {
                response.kind(serenity::model::interactions::InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(format!("thanks for saying maybe, {}", interaction.user)))
            })
            .await
            {
                warn!("Error handling invocation: {}", why);
            } else {
                ctx.session().await.write().await.users.insert(interaction.user.id, crate::session::UserState::MayJoin);
            }
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
        if let Err(why) = interaction
            .create_interaction_response(&ctx.http, |response| {
                response.kind(serenity::model::interactions::InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(format!("thanks for saying no, {}", interaction.user)))
            })
            .await
        {
            warn!("Error handling invocation: {}", why);
        } else {
            ctx.session().await.write().await.users.insert(interaction.user.id, crate::session::UserState::WontJoin);
        }
    }
}
