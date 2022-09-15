use std::{env, fs, sync::Arc};

use dotenv::dotenv;
use loon::Config;
use teloxide::{
    dispatching::{Dispatcher, UpdateHandler},
    dptree,
    payloads::SetWebhookSetters,
    requests::{Requester, RequesterExt},
    Bot,
};
use url::Url;

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

    let token = env::var("BOT_TOKEN").expect("Missing bot token env variable");
    let db_url = env::var("DB_URL").expect("Missing database url env variable");

    let bot = Bot::new(token).auto_send().cache_me();

    if let Ok(u) = env::var("WEBHOOK_URL") {
        let url = Url::parse(&u).expect("Invalid webhook URL");

        bot.set_webhook(url)
            .drop_pending_updates(true)
            .max_connections(100)
            .await
            .expect("Error setting webhook");
    };

    let db = Database::new(db_url).await;
    let loc_dict = Arc::new(
        Config::default()
            .with_path_pattern("locales/*.yml")
            .finish()
            .expect("Can not load localization"),
    );
    let locales = fs::read_dir("locales/")
        .expect("Can not open locales directory")
        .into_iter()
        .filter(|p| p.is_ok())
        .map(|p| p.unwrap().file_name().into_string().unwrap())
        .filter(|s| s.contains(".yml"))
        .map(|s| s.split(".yml").next().unwrap().into())
        .collect::<Vec<Locale>>();

    let x = db.clone();
    let y = bot.clone();

    tokio::spawn(async move {
        loop {
            let ts: i64 = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .try_into()
                .unwrap();

            if let Ok(l) = x.get_pending_messages_to_delete(ts).await {
                for m in l.into_iter() {
                    y.delete_message(m.chat_id.to_string(), m.message_id)
                        .await
                        .ok();
                    x.remove_from_scheduled_delete(m.id).await.ok();
                }
            };

            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    });

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![db, loc_dict, locales])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
