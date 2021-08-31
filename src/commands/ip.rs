use crate::context_ext::ContextExt;

use super::interaction_handler::{CommandHandler, InteractionHandler};
use serenity::{
    async_trait,
    client::Context,
    model::interactions::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
};
use tracing::log::warn;

#[derive(Clone, Copy)]
pub struct Ip;

impl InteractionHandler for Ip {
    fn name(&self) -> &'static str {
        "ip"
    }
}

#[async_trait]
impl CommandHandler for Ip {
    async fn invoke(&self, ctx: Context, interaction: ApplicationCommandInteraction) {
        let content = ctx.config().await.ip_message;
        if let Err(why) = interaction
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.content(content))
            })
            .await
        {
            warn!("Error responding to slash command: {}", why);
        }
    }

    fn create_command(
        self,
        command: &mut serenity::builder::CreateApplicationCommand,
    ) -> &mut serenity::builder::CreateApplicationCommand {
        command
            .name(self.name())
            .description("Shows the IPs currently in use")
    }
}
