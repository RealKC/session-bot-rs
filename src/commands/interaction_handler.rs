use serenity::{
    async_trait,
    builder::CreateApplicationCommand,
    client::Context,
    model::{
        id::GuildId,
        interactions::{application_command::ApplicationCommand, Interaction},
    },
    prelude::{RwLock, TypeMapKey},
};
use std::{collections::HashMap, sync::Arc};

#[async_trait]
pub trait InteractionHandler {
    async fn invoke(&self, ctx: Context, interaction: Interaction);

    fn name(&self) -> &'static str;
}

pub trait Command: InteractionHandler {
    fn create_command(
        self,
        command: &mut CreateApplicationCommand,
    ) -> &mut CreateApplicationCommand;
}

#[derive(Clone)]
pub struct InteractionMap;

impl TypeMapKey for InteractionMap {
    type Value = Arc<RwLock<HashMap<&'static str, Arc<dyn InteractionHandler + Send + Sync>>>>;
}

pub async fn register_global_command<T>(ctx: Context, handler: T)
where
    T: Command + Send + Sync + Copy + 'static,
{
    ApplicationCommand::create_global_application_command(&ctx.http, move |f| {
        handler.create_command(f)
    })
    .await
    .expect(
        format!(
            "There was an error creating global {} command",
            handler.name()
        )
        .as_str(),
    );

    register_interaction_handler(ctx, handler).await;
}

pub async fn register_guild_command<T>(ctx: Context, guild_id: u64, handler: T)
where
    T: Command + Send + Sync + Copy + 'static,
{
    GuildId(guild_id)
        .create_application_command(&ctx.http, |f| handler.create_command(f))
        .await
        .expect(
            format!(
                "There was an error creating guild #{} {} command",
                guild_id,
                handler.name()
            )
            .as_str(),
        );

    register_interaction_handler(ctx, handler).await;
}

pub async fn register_interaction_handler<T>(ctx: Context, handler: T)
where
    T: InteractionHandler + Send + Sync + Copy + 'static,
{
    ctx.data
        .read()
        .await
        .get::<InteractionMap>()
        .expect("There was an error retrieving the InteractionMap")
        .write()
        .await
        .insert(handler.name(), Arc::new(handler.clone()));
}
