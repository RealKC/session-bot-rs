use crate::{commands::prelude::interaction_respond_with_private_message, context_ext::ContextExt};

use super::interaction_handler::{CommandHandler, InteractionHandler};
use serenity::{
    async_trait,
    client::Context,
    model::interactions::{application_command::ApplicationCommandInteraction, Interaction},
};

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
        if !ctx.is_session_running().await {
            interaction_respond_with_private_message(
                ctx.clone(),
                Interaction::ApplicationCommand(interaction),
                "No session running!",
            )
            .await;
            return;
        }

        let user_id = interaction.user.id.clone();
        let host = ctx.session().await.read().await.host.clone();

        if user_id != host && !ctx.config().await.admins.contains(&user_id) {
            interaction_respond_with_private_message(
                ctx.clone(),
                Interaction::ApplicationCommand(interaction),
                "You don't have permissions to cancel this session!",
            )
            .await;
            return;
        }

        // TODO: Actually cancel the session
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
