use std::{fs::read_dir, sync::Arc};

use dotenv::dotenv;
use loon::Config;
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
use crate::types::Locale;

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    teloxide::dptree::entry()
        .branch(settings_handler())
        .branch(setup_poll_handler())
        .branch(vote_yes_handler())
        .branch(vote_no_handler())
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = std::env::var("BOT_TOKEN").expect("Missing bot token env variable");
    let db_url = std::env::var("DB_URL").expect("Missing database url env variable");

    let bot = Bot::new(token).auto_send().cache_me();
    let db = Database::new(db_url).await;
    let loc_dict = Arc::new(
        Config::default()
            .with_path_pattern("locales/*.yml")
            .finish()
            .expect("Can not load localization"),
    );
    let locales = read_dir("locales/")
        .expect("Can not open locales directory")
        .into_iter()
        .filter(|p| p.is_ok())
        .map(|p| p.unwrap().file_name().into_string().unwrap())
        .filter(|s| s.contains(".yml"))
        .map(|s| s.split(".yml").next().unwrap().into())
        .collect::<Vec<Locale>>();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![db, loc_dict, locales])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
