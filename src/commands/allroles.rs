use super::prelude::*;
use crate::context_ext::ContextExt;
use crate::interaction_handler::{CommandHandler, InteractionHandler};

use serenity::model::interactions::application_command::ApplicationCommandOptionType;
use serenity::{
    async_trait, client::Context,
    model::interactions::application_command::ApplicationCommandInteraction,
};

#[derive(Clone, Copy)]
pub struct AllRoles;

impl InteractionHandler for AllRoles {
    fn name(&self) -> &'static str {
        "allroles"
    }
}

#[async_trait]
impl CommandHandler for AllRoles {
    async fn invoke(&self, ctx: Context, interaction: ApplicationCommandInteraction) {
        if ctx.config().await.admins.contains(&interaction.user.id) {
            let (user_id, _) = interaction
                .data
                .resolved
                .members
                .iter()
                .next()
                .expect("Error retrieving user id");

            let guild_id = interaction
                .guild_id
                .clone()
                .expect("Error retrieving guild_id");

            match ctx.http.get_member(guild_id.0, user_id.0).await {
                Ok(mut member) => {
                    let mut role_vector: Vec<RoleId> = ctx
                        .config()
                        .await
                        .games
                        .iter()
                        .map(|game| game.role_id)
                        .filter(|role_id| !member.roles.contains(role_id))
                        .collect();

                    if let Some(role_id) = ctx.config().await.default_user_role {
                        if !member.roles.contains(&role_id) {
                            role_vector.push(role_id);
                        }
                    }

                    match member.add_roles(&ctx.http, &role_vector).await {
                        Ok(_) => {
                            interaction_respond_with_private_message(
                                &ctx,
                                &Interaction::ApplicationCommand(interaction),
                                "Roles added successfully!",
                            )
                            .await
                        }
                        Err(why) => {
                            warn!("There was an error adding the roles: {}", why);
                        }
                    }
                }
                Err(why) => warn!("Error retrieving member: {}", why),
            }
        } else {
            interaction_respond_with_private_message(
                &ctx,
                &Interaction::ApplicationCommand(interaction),
                "You do not have permissions to use this command!",
            )
            .await;
        }
    }

    fn create_command(
        self,
        command: &mut serenity::builder::CreateApplicationCommand,
    ) -> &mut serenity::builder::CreateApplicationCommand {
        command
            .name(self.name())
            .description("Gives verified role + all game roles to a user")
            .create_option(|option| {
                option
                    .kind(ApplicationCommandOptionType::User)
                    .name("user")
                    .description("User to give all the roles to")
                    .required(true)
            })
    }
}
