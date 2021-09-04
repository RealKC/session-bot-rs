use crate::context_ext::ContextExt;

use super::{
    interaction_handler::{InteractionHandler, MessageHandler},
    prelude::*,
};
use serenity::{
    async_trait,
    client::Context,
    model::interactions::{message_component::MessageComponentInteraction, Interaction},
};

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
            .insert(user_id, crate::session::UserState::WillJoin);
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
            .insert(user_id, crate::session::UserState::MayJoin);
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
            .insert(user_id, crate::session::UserState::WontJoin);
    }
}
