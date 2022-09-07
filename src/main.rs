use teloxide::{
    dispatching::{Dispatcher, UpdateHandler},
    dptree,
    requests::RequesterExt,
    Bot,
};

mod database;
mod handlers;
mod types;

use crate::database::Database;
use crate::handlers::{settings_handler, setup_poll_handler, vote_no_handler, vote_yes_handler};

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    teloxide::dptree::entry()
        .branch(settings_handler())
        .branch(setup_poll_handler())
        .branch(vote_yes_handler())
        .branch(vote_no_handler())
}

#[tokio::main]
async fn main() {
    let bot = Bot::from_env().auto_send().cache_me();
    let db = Database::new().await;

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![db])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
