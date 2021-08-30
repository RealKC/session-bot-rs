use crate::config::ContextExt;

use super::interaction_handler::InteractionHandler;
use serenity::{async_trait, client::Context, model::interactions::Interaction};
use tracing::log::warn;

#[derive(Clone, Copy)]
pub struct ButtonYes;

#[derive(Clone, Copy)]
pub struct ButtonMaybe;

#[derive(Clone, Copy)]
pub struct ButtonNo;

#[async_trait]
impl InteractionHandler for ButtonYes {
    fn name(&self) -> &'static str {
        "button-yes"
    }

    async fn invoke(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::MessageComponent(interaction) = interaction {
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
}

#[async_trait]
impl InteractionHandler for ButtonMaybe {
    fn name(&self) -> &'static str {
        "button-maybe"
    }

    async fn invoke(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::MessageComponent(interaction) = interaction {
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
}

#[async_trait]
impl InteractionHandler for ButtonNo {
    fn name(&self) -> &'static str {
        "button-no"
    }

    async fn invoke(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::MessageComponent(interaction) = interaction {
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
}
