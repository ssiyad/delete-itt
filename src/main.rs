use std::{collections::HashMap, sync::Arc};

use teloxide::{
    adaptors::AutoSend,
    dispatching::{Dispatcher, UpdateFilterExt, UpdateHandler},
    dptree,
    payloads::SendMessageSetters,
    requests::{Requester, RequesterExt},
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message, Update},
    Bot,
};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
struct PollInformation {
    chat_id: i64,
    poll_id: i32,
    message_id: i32,
    minimum_vote_count: u8,
    vote_count_yes: u8,
    vote_count_no: u8,
}

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
type Storage = Arc<Mutex<HashMap<String, PollInformation>>>;

fn gen_combined_id(chat_id: i64, message_id: i32) -> String {
    format!("{}{}", chat_id, message_id)
}

async fn get_from_storage(
    storage: &Storage,
    chat_id: i64,
    message_id: i32,
) -> Option<PollInformation> {
    let s = storage.lock().await;
    s.get(&gen_combined_id(chat_id, message_id)).cloned()
}

async fn put_into_storage(
    storage: &Storage,
    chat_id: i64,
    message_id: i32,
    data: PollInformation,
) -> Option<PollInformation> {
    let mut s = storage.lock().await;
    s.insert(gen_combined_id(chat_id, message_id), data)
}

async fn remove_from_storage(
    storage: &Storage,
    chat_id: i64,
    message_id: i32,
) -> Option<PollInformation> {
    let mut s = storage.lock().await;
    s.remove(&gen_combined_id(chat_id, message_id))
}

async fn target_me(bot: AutoSend<Bot>, msg: Message) -> bool {
    if let Some(t) = msg.text() {
        let me = bot.get_me().await.unwrap();
        let username = me.username.as_ref().unwrap();

        if t.len() == username.len() + 1 {
            t.starts_with(&format!("@{}", username))
        } else {
            t.starts_with(&format!("@{} ", username))
        }
    } else {
        false
    }
}

async fn setup_poll(bot: AutoSend<Bot>, msg: Message, storage: Storage) -> HandlerResult {
    if let Some(reply_to_message_id) = msg.reply_to_message() {
        bot.delete_message(msg.chat.id, msg.id).await?;

        let markup = InlineKeyboardMarkup::default().append_row(vec![
            InlineKeyboardButton::callback("Yes", "vote_yes"),
            InlineKeyboardButton::callback("No", "vote_no"),
        ]);

        let poll_msg = bot
            .send_message(
                msg.chat.id,
                "Should I delete this message? Minimum number of vote needed is 5",
            )
            .reply_to_message_id(reply_to_message_id.id)
            .reply_markup(markup)
            .await?;

        let info = PollInformation {
            chat_id: msg.chat.id.0,
            poll_id: poll_msg.id,
            message_id: reply_to_message_id.id,
            minimum_vote_count: 5,
            vote_count_yes: 0,
            vote_count_no: 0,
        };

        put_into_storage(&storage, msg.chat.id.0, poll_msg.id, info).await;
    }

    Ok(())
}

async fn handle_vote_yes(
    bot: AutoSend<Bot>,
    query: CallbackQuery,
    storage: Storage,
) -> HandlerResult {
    let msg = query.message.unwrap();

    if let Some(mut info) = get_from_storage(&storage, msg.chat.id.0, msg.id).await {
        info.vote_count_yes += 1;

        if info.vote_count_yes == info.minimum_vote_count {
            bot.delete_message(info.chat_id.to_string(), info.message_id)
                .await
                .unwrap();

            bot.delete_message(info.chat_id.to_string(), info.poll_id)
                .await?;

            remove_from_storage(&storage, msg.chat.id.0, msg.id).await;
        } else {
            put_into_storage(&storage, msg.chat.id.0, msg.id, info).await;
        }
    };

    Ok(())
}

async fn handle_vote_no(
    bot: AutoSend<Bot>,
    query: CallbackQuery,
    storage: Storage,
) -> HandlerResult {
    let msg = query.message.unwrap();

    if let Some(mut info) = get_from_storage(&storage, msg.chat.id.0, msg.id).await {
        info.vote_count_no += 1;

        if info.vote_count_no == info.minimum_vote_count {
            bot.delete_message(info.chat_id.to_string(), info.poll_id)
                .await?;

            remove_from_storage(&storage, msg.chat.id.0, msg.id).await;
        } else {
            put_into_storage(&storage, msg.chat.id.0, msg.id, info).await;
        }
    };

    Ok(())
}

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
