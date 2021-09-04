use crate::{
    commands::{prelude::*, status::users_with_state},
    context_ext::ContextExt,
    session::{Session, UserState},
};

use super::interaction_handler::{CommandHandler, InteractionHandler, MessageHandler};
use serenity::{
    async_trait,
    builder::CreateActionRow,
    client::Context,
    model::{
        id::UserId,
        interactions::{
            application_command::ApplicationCommandInteraction,
            message_component::{ButtonStyle, MessageComponentInteraction},
            Interaction,
        },
    },
};

fn get_action_row() -> CreateActionRow {
    CreateActionRow::default()
        .create_button(|button| {
            button
                .style(ButtonStyle::Danger)
                .label("Yes")
                .custom_id("endhost-yes")
        })
        .create_button(|button| {
            button
                .style(ButtonStyle::Success)
                .label("No")
                .custom_id("endhost-no")
        })
        .to_owned()
}

async fn can_cancel_session(ctx: &Context, user_id: UserId) -> bool {
    let host = ctx.session().await.read().await.host.clone();
    user_id == host || ctx.config().await.admins.contains(&user_id)
}

#[derive(Clone, Copy)]
pub struct EndHost;

impl InteractionHandler for EndHost {
    fn name(&self) -> &'static str {
        "endhost"
    }
}

#[async_trait]
impl CommandHandler for EndHost {
    async fn invoke(&self, ctx: Context, interaction: ApplicationCommandInteraction) {
        if !ctx.is_session_present().await {
            interaction_respond_with_private_message(
                &ctx,
                &Interaction::ApplicationCommand(interaction),
                "No session currently running!",
            )
            .await;
            return;
        }

        let user_id = interaction.user.id;
        if !can_cancel_session(&ctx, user_id).await {
            interaction_respond_with_private_message(
                &ctx,
                &Interaction::ApplicationCommand(interaction),
                "You don't have permissions to cancel this session!",
            )
            .await;
            return;
        }

        let action = if ctx.is_session_started().await {
            "end"
        } else {
            "cancel"
        };

        interaction
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .content(format!("Are you sure you want to {} the Session?", action))
                            .components(|components| components.add_action_row(get_action_row()))
                            .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                    })
            })
            .await
            .unwrap_or_else(|why| warn!("Error responding to interaction: {}", why));
    }

    fn create_command(
        self,
        command: &mut serenity::builder::CreateApplicationCommand,
    ) -> &mut serenity::builder::CreateApplicationCommand {
        command
            .name(self.name())
            .description("Ends/Cancels the current session")
    }
}

#[derive(Clone, Copy)]
pub struct EndHostButtonYes;

#[derive(Clone, Copy)]
pub struct EndHostButtonNo;

impl InteractionHandler for EndHostButtonYes {
    fn name(&self) -> &'static str {
        "endhost-yes"
    }
}

#[async_trait]
impl MessageHandler for EndHostButtonYes {
    async fn invoke(&self, ctx: Context, interaction: MessageComponentInteraction) {
        if !ctx.is_session_present().await {
            interaction_respond_with_private_message(
                &ctx,
                &Interaction::MessageComponent(interaction),
                "No session currently running!",
            )
            .await;
            return;
        }

        let action = if ctx.is_session_started().await {
            "ended"
        } else {
            "cancelled"
        };
        if !can_cancel_session(&ctx, interaction.user.id).await {
            interaction_respond_with_private_message(
                &ctx,
                &Interaction::MessageComponent(interaction),
                format!("You don't have permission to {} this session!", action).as_str(),
            )
            .await;
            return;
        }

        let message_id = ctx.session().await.read().await.message_id;
        let game = ctx.session().await.read().await.game.clone();
        let channel_id = game.channel_id;

        if let Ok(message) = ctx
            .http
            .get_message(channel_id.0, message_id.0)
            .await
            .as_mut()
        {
            message
                .edit(&ctx, |message| {
                    message.components(|components| components.set_action_rows(vec![]))
                })
                .await
                .unwrap_or_else(|why| warn!("Error editing message: {}", why));

            message
                .unpin(&ctx)
                .await
                .unwrap_or_else(|why| warn!("Error unpinning message: {}", why));
        }

        let content = if !ctx.is_session_started().await {
            let user_pings =
                users_with_state(&ctx.session().await.read().await.users, UserState::WillJoin);
            if user_pings.1 == 0 {
                "".to_string()
            } else {
                user_pings.0 + ": "
            }
        } else {
            "".to_string()
        };

        if let Err(why) = channel_id
            .send_message(&ctx.http, |message| {
                message.content(format!(
                    "{}{} Session has been {}!",
                    content, game.name, action
                ))
            })
            .await
        {
            warn!("Error sending message: {}", why);
        }

        ctx.session().await.write().await.handle.abort();
        ctx.data.write().await.remove::<Session>();
        update_bot_status(&ctx).await;

        interaction
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|message| {
                        message
                            .content(format!("Session *has* been {}!", action))
                            .components(|components| components.set_action_rows(vec![]))
                    })
            })
            .await
            .unwrap_or_else(|why| warn!("Error responding to interaction: {}", why));
    }
}

impl InteractionHandler for EndHostButtonNo {
    fn name(&self) -> &'static str {
        "endhost-no"
    }
}

#[async_trait]
impl MessageHandler for EndHostButtonNo {
    async fn invoke(&self, ctx: Context, interaction: MessageComponentInteraction) {
        let action = if ctx.is_session_started().await {
            "ended"
        } else {
            "cancelled"
        };

        interaction
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|message| {
                        message
                            .content(format!("Session has *not* been {}!", action))
                            .components(|f| f.set_action_rows(vec![]))
                    })
            })
            .await
            .unwrap_or_else(|why| warn!("Error responding to interaction: {}", why));
    }
}
