use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Local};
use serenity::{
    model::id::{MessageId, UserId},
    prelude::{RwLock, TypeMapKey},
};
use tokio::task::JoinHandle;

use crate::config::Game;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserState {
    Will,
    May,
    Wont,
}

pub struct Session {
    pub game: Game,
    pub users: HashMap<UserId, UserState>,
    pub time: DateTime<Local>,
    pub handle: JoinHandle<()>,
    pub message_id: MessageId,
    pub host: UserId,
}

impl TypeMapKey for Session {
    type Value = Arc<RwLock<Session>>;
}

impl Session {
    pub fn new(
        game: Game,
        handle: JoinHandle<()>,
        time: DateTime<Local>,
        message_id: MessageId,
        host: UserId,
    ) -> Self {
        Self {
            game,
            users: HashMap::new(),
            time,
            handle,
            message_id,
            host,
        }
    }
}
