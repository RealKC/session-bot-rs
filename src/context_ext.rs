use std::{collections::HashMap, sync::Arc};

use chrono::Local;
use serenity::{async_trait, client::Context, prelude::RwLock};

use crate::{
    commands::interaction_handler::{Handler, InteractionMap},
    config::Config,
    session::Session,
};

#[async_trait]
pub trait ContextExt {
    async fn config(&self) -> Config;
    async fn session(&self) -> Arc<RwLock<Session>>;
    async fn is_session_present(&self) -> bool;
    async fn is_session_started(&self) -> bool;
    async fn interaction_map(&self) -> HashMap<&'static str, Handler>;
}

#[async_trait]
impl ContextExt for Context {
    async fn config(&self) -> Config {
        self.data
            .read()
            .await
            .get::<Config>()
            .expect("Error reading config from TypeMap")
            .read()
            .await
            .clone()
    }

    async fn session(&self) -> Arc<RwLock<Session>> {
        self.data
            .read()
            .await
            .get::<Session>()
            .expect("Error reading session from TypeMap")
            .clone()
    }

    async fn is_session_present(&self) -> bool {
        self.data.read().await.get::<Session>().is_some()
    }

    async fn is_session_started(&self) -> bool {
        self.is_session_present().await && self.session().await.read().await.time <= Local::now()
    }

    async fn interaction_map(&self) -> HashMap<&'static str, Handler> {
        self.data
            .read()
            .await
            .get::<InteractionMap>()
            .expect("There was an error retrieving the InteractionMap")
            .read()
            .await
            .clone()
    }
}
