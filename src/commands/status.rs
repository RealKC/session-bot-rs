use crate::{context_ext::ContextExt, session::UserState};

use super::interaction_handler::{CommandHandler, InteractionHandler};
use serenity::{
    async_trait, client::Context,
    model::interactions::application_command::ApplicationCommandInteraction, utils::Colour,
};
use tracing::log::warn;

#[derive(Clone, Copy)]
pub struct Status;

impl InteractionHandler for Status {
    fn name(&self) -> &'static str {
        "status"
    }
}

#[async_trait]
impl CommandHandler for Status {
    async fn invoke(&self, ctx: Context, interaction: ApplicationCommandInteraction) {
        let user_map = ctx.session().await.read().await.users.clone();
        let will_join = user_map
            .iter()
            .filter(|(_, state)| **state == UserState::WillJoin)
            .fold(String::new(), |a, (b, _)| a + format!("<@{}> ", b).as_str());
        let may_join = user_map
            .iter()
            .filter(|(_, state)| **state == UserState::MayJoin)
            .fold(String::new(), |a, (b, _)| a + format!("<@{}> ", b).as_str());
        let wont_join = user_map
            .iter()
            .filter(|(_, state)| **state == UserState::WontJoin)
            .fold(String::new(), |a, (b, _)| a + format!("<@{}> ", b).as_str());

        if let Err(why) = interaction
            .create_interaction_response(&ctx.http, |response| {
                response.kind(serenity::model::interactions::InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.create_embed(|embed| {
                        embed.title("Status").colour(Colour::from_rgb(244, 173, 249))
                            .field("People who are sure", if will_join.is_empty() {String::from("Nobody")} else {will_join}, false)
                            .field("People who are unsure",if may_join.is_empty() {String::from("Nobody")} else {may_join}, false)
                            .field("People who don't want to",if wont_join.is_empty() {String::from("Nobody")} else {wont_join}, false)
                    })
                })
            })
            .await
        {
            warn!("Error handling invocation: {}", why);
        }
    }

    fn create_command(
        self,
        command: &mut serenity::builder::CreateApplicationCommand,
    ) -> &mut serenity::builder::CreateApplicationCommand {
        command
            .name(self.name())
            .description("Status of the current game session")
    }
}
