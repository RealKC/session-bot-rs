use super::interaction_handler::InteractionHandler;
use serenity::{
    async_trait,
    client::Context,
    model::interactions::{Interaction, InteractionResponseType},
};
use tracing::log::warn;

#[derive(Clone, Copy)]
pub struct Ping;

#[async_trait]
impl InteractionHandler for Ping {
    fn name(&self) -> &'static str {
        "ping"
    }

    async fn invoke(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(interaction) = interaction {
            if let Err(why) = interaction
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content("Pong lmao"))
                })
                .await
            {
                warn!("Error responding to slash command: {}", why);
            }
        }
    }

    fn create_command(
        self,
        command: &mut serenity::builder::CreateApplicationCommand,
    ) -> &mut serenity::builder::CreateApplicationCommand {
        command.name(self.name()).description("A ping/pong command")
    }
}
