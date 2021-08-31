use serenity::{
    async_trait,
    builder::CreateApplicationCommand,
    client::Context,
    model::{
        id::GuildId,
        interactions::{
            application_command::{ApplicationCommand, ApplicationCommandInteraction},
            message_component::MessageComponentInteraction,
        },
    },
    prelude::{RwLock, TypeMapKey},
};
use std::{collections::HashMap, sync::Arc};

pub trait InteractionHandler {
    fn name(&self) -> &'static str;
}

#[async_trait]
pub trait CommandHandler: InteractionHandler {
    async fn invoke(&self, ctx: Context, interaction: ApplicationCommandInteraction);

    fn create_command(
        self,
        command: &mut CreateApplicationCommand,
    ) -> &mut CreateApplicationCommand;
}

#[async_trait]
pub trait MessageHandler: InteractionHandler {
    async fn invoke(&self, ctx: Context, interaction: MessageComponentInteraction);
}

#[derive(Clone)]
pub enum Handler {
    Command(Arc<dyn CommandHandler + Send + Sync>),
    Message(Arc<dyn MessageHandler + Send + Sync>),
}

#[derive(Clone)]
pub struct InteractionMap;

impl TypeMapKey for InteractionMap {
    type Value = Arc<RwLock<HashMap<&'static str, Handler>>>;
}

pub async fn register_global_command<T>(ctx: Context, handler: T)
where
    T: CommandHandler + Send + Sync + Copy + 'static,
{
    ApplicationCommand::create_global_application_command(&ctx.http, move |f| {
        handler.create_command(f)
    })
    .await
    .unwrap_or_else(|_| {
        panic!(
            "There was an error creating global {} command",
            handler.name()
        )
    });

    register_handler(ctx, Handler::Command(Arc::new(handler))).await;
}

pub async fn register_guild_command<T>(ctx: Context, guild_id: u64, handler: T)
where
    T: CommandHandler + Send + Sync + Copy + 'static,
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

    register_handler(ctx, Handler::Command(Arc::new(handler))).await;
}

pub async fn register_handler(ctx: Context, handler: Handler) {
    let name = match &handler {
        Handler::Command(command) => command.name(),
        Handler::Message(message) => message.name(),
    };

    ctx.data
        .read()
        .await
        .get::<InteractionMap>()
        .expect("There was an error retrieving the InteractionMap")
        .write()
        .await
        .insert(name, handler);
}
