use std::collections::HashMap;

use crate::{context_ext::ContextExt, session::UserState};

use super::interaction_handler::{CommandHandler, InteractionHandler};
use serenity::{
    async_trait,
    client::Context,
    model::{
        id::UserId,
        interactions::{
            application_command::ApplicationCommandInteraction,
            InteractionApplicationCommandCallbackDataFlags,
        },
    },
    utils::Colour,
};
use tracing::log::warn;

#[derive(Clone, Copy)]
pub struct Status;

impl InteractionHandler for Status {
    fn name(&self) -> &'static str {
        "status"
    }
}

pub fn users_with_state(user_map: &HashMap<UserId, UserState>, state: UserState) -> String {
    let ans = user_map
        .iter()
        .filter(|(_, s)| **s == state)
        .fold(String::new(), |lhs, (rhs, _)| {
            lhs + format!("<@{}> ", rhs).as_str()
        });

    if ans.is_empty() {
        "Nobody".to_string()
    } else {
        ans
    }
}

#[async_trait]
impl CommandHandler for Status {
    async fn invoke(&self, ctx: Context, interaction: ApplicationCommandInteraction) {
        let user_map = ctx.session().await.read().await.users.clone();
        let will_join = users_with_state(&user_map, UserState::WillJoin);
        let may_join = users_with_state(&user_map, UserState::MayJoin);
        let wont_join = users_with_state(&user_map, UserState::WontJoin);

        let res  = interaction
            .create_interaction_response(&ctx.http, |response| {
                response.kind(serenity::model::interactions::InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.create_embed(|embed| {
                        embed.title("Status").colour(Colour::from_rgb(244, 173, 249))
                            .field("People who are sure", will_join, false)
                            .field("People who are unsure",may_join, false)
                            .field("People who don't want to",wont_join, false)
                    })
                    .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                })
            })
            .await;

        if let Err(why) = res {
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
