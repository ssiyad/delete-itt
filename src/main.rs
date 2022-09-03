use std::{collections::HashMap, sync::Arc};

use teloxide::{
    dispatching::{Dispatcher, UpdateHandler},
    dptree,
    requests::RequesterExt,
    Bot,
};
use tokio::sync::Mutex;

mod handlers;
mod storage;
mod types;
mod utils;

use crate::handlers::{setup_poll_handler, vote_no_handler, vote_yes_handler};
use crate::types::Storage;

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    teloxide::dptree::entry()
        .branch(setup_poll_handler())
        .branch(vote_yes_handler())
        .branch(vote_no_handler())
}

#[tokio::main]
async fn main() {
    let bot = Bot::from_env().auto_send();
    let storage: Storage = Arc::new(Mutex::new(HashMap::new()));

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![storage])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
