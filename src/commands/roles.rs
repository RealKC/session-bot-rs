use crate::{
    commands::prelude::*,
    config::Game,
    context_ext::ContextExt,
    interaction_handler::{CommandHandler, InteractionHandler, MessageHandler},
};

use serenity::{
    async_trait,
    builder::{CreateActionRow, CreateSelectMenuOption},
    client::Context,
    model::interactions::{
        application_command::ApplicationCommandInteraction,
        message_component::MessageComponentInteraction,
    },
};

async fn get_select_menu_option(
    ctx: &Context,
    game: &Game,
    member: &Member,
    idx: usize,
) -> Option<CreateSelectMenuOption> {
    let role_id = game.role_id;

    if let Some(roles) = member.roles(&ctx.cache).await {
        let is_set = roles.iter().any(|role| role.id == role_id);
        let is_set = if is_set { "" } else { "not " };

        Some(
            CreateSelectMenuOption::default()
                .label(&game.name)
                .description(format!("This role is {}set", is_set))
                .value(idx)
                .to_owned(),
        )
    } else {
        None
    }
}

async fn get_action_row(ctx: &Context, member: &Member) -> CreateActionRow {
    let mut options_vec = vec![];

    for (idx, game) in ctx.config().await.games.iter().enumerate() {
        if let Some(option) = get_select_menu_option(&ctx, &game, &member, idx).await {
            options_vec.push(option);
        }
    }

    CreateActionRow::default()
        .create_select_menu(|menu| {
            menu.custom_id("roles-dropdown")
                .options(|options| options.set_options(options_vec))
        })
        .clone()
}

#[derive(Clone, Copy)]
pub struct RolesCommand;

impl InteractionHandler for RolesCommand {
    fn name(&self) -> &'static str {
        "roles"
    }
}

#[async_trait]
impl CommandHandler for RolesCommand {
    async fn invoke(&self, ctx: Context, interaction: ApplicationCommandInteraction) {
        if let Some(member) = &interaction.member {
            let action_row = get_action_row(&ctx, &member).await;
            interaction
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message
                                .content("Select which role to add/remove!")
                                .components(|components| components.add_action_row(action_row))
                                .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                        })
                })
                .await
                .unwrap_or_else(|why| warn!("Error responding to interaction: {}", why));
        }
    }

    fn create_command(
        self,
        command: &mut serenity::builder::CreateApplicationCommand,
    ) -> &mut serenity::builder::CreateApplicationCommand {
        command.name(self.name()).description("Adds/removes roles")
    }
}

#[derive(Clone, Copy)]
pub struct RolesCommandHandler;

impl InteractionHandler for RolesCommandHandler {
    fn name(&self) -> &'static str {
        "roles-dropdown"
    }
}

#[async_trait]
impl MessageHandler for RolesCommandHandler {
    async fn invoke(&self, ctx: Context, interaction: MessageComponentInteraction) {
        // The conversion should always be valid unless a request is forged via modifications
        // This is due to the fact .values[0] will always be a value set via get_action_row()
        let index = interaction.clone().data.values[0]
            .parse::<usize>()
            .expect("Error parsing role data to usize");

        let role_id = ctx.config().await.games[index].role_id;
        if let Some(member) = &mut interaction.member.clone() {
            if let Some(roles) = member.roles(&ctx.cache).await {
                let action = if roles.iter().any(|role| role.id == role_id) {
                    member
                        .remove_role(&ctx.http, role_id)
                        .await
                        .unwrap_or_else(|why| warn!("Error removing role: {}", why));
                    "un"
                } else {
                    member
                        .add_role(&ctx.http, role_id)
                        .await
                        .unwrap_or_else(|why| warn!("Error adding role: {}", why));
                    ""
                };

                let action_row = get_action_row(&ctx, &member).await;
                interaction
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::UpdateMessage)
                            .interaction_response_data(|message| {
                                message
                                    .content(format!(
                                        "Role <@&{}> has been {}set!",
                                        role_id, action
                                    ))
                                    .components(|components| components.add_action_row(action_row))
                            })
                    })
                    .await
                    .unwrap_or_else(|why| warn!("Error responding to interaction: {}", why));
            } else {
                warn!("Error retrieving roles");
            }
        } else {
            warn!("Error retrieving member");
        }
    }
}
