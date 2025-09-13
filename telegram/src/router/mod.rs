mod callback;
mod commands;
mod state;
mod text_commands;

use anyhow::{Error, Result};
use teloxide::{
    dispatching::{
        DpHandlerDescription, UpdateHandler,
        dialogue::{self, InMemStorage},
    },
    prelude::*,
};

use crate::State;

pub fn master() -> UpdateHandler<Error> {
    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_router())
        .branch(callback::router())
}
fn message_router() -> Handler<'static, Result<()>, DpHandlerDescription> {
    Update::filter_message()
        .branch(commands::router())
        .branch(text_commands::router())
        .branch(state::router())
}
