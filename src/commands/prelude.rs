pub use serenity::{
    client::Context,
    model::interactions::{
        Interaction, InteractionApplicationCommandCallbackDataFlags, InteractionResponseType,
    },
};
pub use tracing::{error, info, warn};

pub async fn interaction_respond_with_private_message(
    ctx: Context,
    interaction: Interaction,
    content: &str,
) {
    match interaction {
        Interaction::ApplicationCommand(interaction) => interaction
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .content(content)
                            .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                    })
            })
            .await
            .unwrap_or_else(|why| warn!("Error responding to interaction: {}", why)),
        Interaction::MessageComponent(interaction) => interaction
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .content(content)
                            .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                    })
            })
            .await
            .unwrap_or_else(|why| warn!("Error responding to interaction: {}", why)),
        Interaction::Ping(_) => warn!("Cant respond to ping interaction!"),
    }
}
