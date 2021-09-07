use crate::{
    commands::prelude::*,
    config::ColorRole,
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
    color_role: &ColorRole,
    idx: usize,
) -> Option<CreateSelectMenuOption> {
    let role_id = color_role.role_id;
    let color = role_id.to_role_cached(&ctx.cache).await?.colour;

    Some(
        CreateSelectMenuOption::default()
            .label(&color_role.name)
            .description(format!("#{}", color.hex()))
            .value(idx)
            .to_owned(),
    )
}

async fn get_action_row(ctx: &Context) -> CreateActionRow {
    let mut options_vec = vec![];

    for (idx, color_role) in ctx.config().await.colors.iter().enumerate() {
        if let Some(option) = get_select_menu_option(&ctx, &color_role, idx).await {
            options_vec.push(option);
        }
    }

    CreateActionRow::default()
        .create_select_menu(|menu| {
            menu.custom_id("colorroles-dropdown")
                .options(|options| options.set_options(options_vec))
        })
        .clone()
}

#[derive(Clone, Copy)]
pub struct ColorsCommand;

impl InteractionHandler for ColorsCommand {
    fn name(&self) -> &'static str {
        "colors"
    }
}

#[async_trait]
impl CommandHandler for ColorsCommand {
    async fn invoke(&self, ctx: Context, interaction: ApplicationCommandInteraction) {
        let action_row = get_action_row(&ctx).await;

        if let Some(member) = &interaction.member {
            let color_roles = &ctx.config().await.colors;
            let role_id = member
                .roles
                .iter()
                .filter(|role| {
                    color_roles
                        .iter()
                        .any(|color_role| color_role.role_id == **role)
                })
                .nth(0);

            let content = if let Some(role_id) = role_id {
                format!("You currently have the <@&{}> color role", role_id)
            } else {
                "No color role currently set, select to add one!".to_string()
            };

            interaction
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message
                                .content(content)
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
pub struct MenuHandler;

impl InteractionHandler for MenuHandler {
    fn name(&self) -> &'static str {
        "colorroles-dropdown"
    }
}

#[async_trait]
impl MessageHandler for MenuHandler {
    async fn invoke(&self, ctx: Context, interaction: MessageComponentInteraction) {
        let index = interaction.clone().data.values[0]
            .parse::<usize>()
            .expect("Error parsing role data to usize");

        let role_id = ctx.config().await.colors[index].role_id;
        let mut member = interaction.member.clone().expect("Error retrieving member");
        let color_roles = &ctx.config().await.colors;
        let roles = member
            .roles(&ctx.cache)
            .await
            .expect("Error retrieving roles");

        let roles_to_remove: Vec<RoleId> = roles
            .iter()
            .filter(|role| {
                color_roles
                    .iter()
                    .any(|color_role| color_role.role_id == role.id)
            })
            .map(|role| role.id)
            .collect();

        if let Err(why) = member.remove_roles(&ctx.http, &roles_to_remove).await {
            warn!("Error removing roles: {}", why);
        }

        if let Err(why) = member.add_role(&ctx.http, role_id).await {
            warn!("Error adding role: {}", why);
        }

        let action_row = get_action_row(&ctx).await;
        interaction
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|message| {
                        message
                            .content(format!("You currently have the <@&{}> color role", role_id))
                            .components(|components| components.add_action_row(action_row))
                    })
            })
            .await
            .unwrap_or_else(|why| warn!("Error responding to interaction: {}", why));
    }
}
