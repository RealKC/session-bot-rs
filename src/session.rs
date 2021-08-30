use std::{collections::HashMap, sync::Arc};

use serenity::{
    model::id::UserId,
    prelude::{RwLock, TypeMapKey},
};
use tokio::task::JoinHandle;

use crate::config::Game;

#[derive(Clone, Copy, Debug)]
pub enum UserState {
    WillJoin,
    MayJoin,
    WontJoin,
}

pub struct Session {
    pub game: Game,
    pub users: HashMap<UserId, UserState>,
    pub will_happen_tomorrow: bool,
    pub handle: JoinHandle<()>,
}

impl TypeMapKey for Session {
    type Value = Arc<RwLock<Session>>;
}

impl Session {
    pub fn new(game: Game, handle: JoinHandle<()>, will_happen_tomorrow: bool) -> Self {
        Self {
            game,
            users: HashMap::new(),
            will_happen_tomorrow,
            handle,
        }
    }
}
