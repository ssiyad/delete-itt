use std::{collections::HashMap, sync::Arc};

use teloxide::{
    dispatching::{Dispatcher, UpdateFilterExt, UpdateHandler},
    dptree,
    requests::RequesterExt,
    types::{CallbackQuery, Update},
    Bot,
};
use tokio::sync::Mutex;

mod handlers;
mod types;
mod storage;
mod utils;

use crate::handlers::{target_me, setup_poll, handle_vote_yes, handle_vote_no};
use crate::types::Storage;

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    let message_handler = Update::filter_message()
        .filter_async(target_me)
        .endpoint(setup_poll);

    let vote_yes_handler = Update::filter_callback_query()
        .filter(|query: CallbackQuery| query.data.unwrap().eq("vote_yes"))
        .endpoint(handle_vote_yes);

    let vote_no_handler = Update::filter_callback_query()
        .filter(|query: CallbackQuery| query.data.unwrap().eq("vote_no"))
        .endpoint(handle_vote_no);

    teloxide::dptree::entry()
        .branch(message_handler)
        .branch(vote_yes_handler)
        .branch(vote_no_handler)
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
