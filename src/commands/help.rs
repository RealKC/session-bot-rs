use crate::{commands::prelude::*, context_ext::ContextExt, embed::Embed};

use super::interaction_handler::{CommandHandler, InteractionHandler, MessageHandler};
use serde::Deserialize;
use serenity::{
    async_trait,
    builder::{CreateActionRow, CreateSelectMenuOption},
    client::Context,
    model::interactions::{
        application_command::ApplicationCommandInteraction,
        message_component::MessageComponentInteraction,
    },
};

#[derive(Deserialize, Clone)]
pub struct HelpPage {
    pub dropdown_title: String,
    pub dropdown_description: String,
    pub embed: Embed,
}

impl HelpPage {
    fn get_option(&self, index: u64) -> CreateSelectMenuOption {
        CreateSelectMenuOption::default()
            .label(&self.dropdown_title)
            .description(&self.dropdown_description)
            .value(index)
            .clone()
    }
}

async fn get_action_row(ctx: &Context) -> CreateActionRow {
    let option_vec = ctx
        .config()
        .await
        .help
        .iter()
        .enumerate()
        .map(|(idx, page)| page.get_option(idx as u64))
        .collect();

    CreateActionRow::default()
        .create_select_menu(|menu| {
            menu.custom_id("help-pages")
                .options(|options| options.set_options(option_vec))
        })
        .clone()
}

#[derive(Clone, Copy)]
pub struct Help;

impl InteractionHandler for Help {
    fn name(&self) -> &'static str {
        "help"
    }
}

#[async_trait]
impl CommandHandler for Help {
    async fn invoke(&self, ctx: Context, interaction: ApplicationCommandInteraction) {
        let embed = ctx.config().await.default_help.to_discord_embed();
        let action_row = get_action_row(&ctx).await;

        interaction
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .add_embed(embed)
                            .components(|components| components.add_action_row(action_row))
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
        command.name(self.name()).description("Shows help pages")
    }
}

#[derive(Clone, Copy)]
pub struct HelpPageHandler;

impl InteractionHandler for HelpPageHandler {
    fn name(&self) -> &'static str {
        "help-pages"
    }
}

#[async_trait]
impl MessageHandler for HelpPageHandler {
    async fn invoke(&self, ctx: Context, interaction: MessageComponentInteraction) {
        // The conversion should always be valid unless a request is forged via modifications
        // This is due to the fact .values[0] will always be one set via HelpPage::get_option
        let index = interaction.clone().data.values[0]
            .parse::<usize>()
            .expect("Error parsing help-page data to usize");

        let embed = ctx.config().await.help[index].embed.to_discord_embed();

        interaction
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|message| message.add_embed(embed))
            })
            .await
            .unwrap_or_else(|why| warn!("Error responding to interaction: {}", why));
    }
}
