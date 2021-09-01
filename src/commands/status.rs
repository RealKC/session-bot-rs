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

fn users_with_state(user_map: &HashMap<UserId, UserState>, state: UserState) -> (String, u64) {
    let ans = user_map
        .iter()
        .filter(|(_, s)| **s == state)
        .fold((String::new(), 0), |lhs, (rhs, _)| {
            (lhs.0 + format!("<@{}> ", rhs).as_str(), lhs.1 + 1)
        });

    if ans.0.is_empty() {
        ("Nobody".to_string(), 0)
    } else {
        ans
    }
}

#[async_trait]
impl CommandHandler for Status {
    async fn invoke(&self, ctx: Context, interaction: ApplicationCommandInteraction) {
        let session = ctx.session().await;
        let user_map = session.read().await.users.clone();
        let host = session
            .read()
            .await
            .host
            .to_user(&ctx.http)
            .await
            .unwrap_or_default();

        let host_nick = host
            .nick_in(&ctx.http, interaction.guild_id.unwrap_or_default())
            .await
            .unwrap_or(host.name.clone());

        let (will_join, will_join_amount) = users_with_state(&user_map, UserState::WillJoin);
        let (may_join, may_join_amount) = users_with_state(&user_map, UserState::MayJoin);
        let (wont_join, wont_join_amount) = users_with_state(&user_map, UserState::WontJoin);

        let res  = interaction
            .create_interaction_response(&ctx.http, |response| {
                response.kind(serenity::model::interactions::InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.create_embed(|embed| {
                        embed
                            .title("Session Status")
                            .colour(Colour::from_rgb(244, 173, 249))
                            .author(|author| author.name(host_nick).icon_url(host.face().clone()))
                            .field(format!("People who are sure: {}", will_join_amount), will_join, false)
                            .field(format!("People who are unsure: {}", may_join_amount),may_join, false)
                            .field(format!("People who dont want to: {}", wont_join_amount),wont_join, false)
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
