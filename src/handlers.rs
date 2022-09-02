use teloxide::{
    adaptors::AutoSend,
    dispatching::UpdateFilterExt,
    payloads::{AnswerCallbackQuerySetters, SendMessageSetters},
    requests::Requester,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message, Update},
    Bot,
};

use crate::storage::{get_from_storage, put_into_storage, remove_from_storage};
use crate::types::{HandlerResult, PollInformation, Storage, AtomicHandler};

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

async fn non_duplicate(
    query: CallbackQuery,
    storage: Storage,
) -> bool {
    let msg = query.message.unwrap();

    if let Some(info) = get_from_storage(&storage, msg.chat.id.0, msg.id).await {
        !info.voters.contains(&query.from.id.0)
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
            voters: vec![],
        };

        put_into_storage(&storage, msg.chat.id.0, poll_msg.id, info).await;
    }

    Ok(())
}

pub fn message_handler() -> AtomicHandler {
    Update::filter_message()
        .filter_async(target_me)
        .endpoint(setup_poll)
}

async fn handle_vote_yes(
    bot: AutoSend<Bot>,
    query: CallbackQuery,
    storage: Storage,
) -> HandlerResult {
    let msg = query.message.unwrap();

    if let Some(mut info) = get_from_storage(&storage, msg.chat.id.0, msg.id).await {
        info.vote_count_yes += 1;
        info.voters.push(query.from.id.0);

        bot.answer_callback_query(query.id)
            .text("You voted to delete the message")
            .await?;

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

pub fn vote_yes_handler() -> AtomicHandler {
    Update::filter_callback_query()
        .filter(|query: CallbackQuery| query.data.unwrap().eq("vote_yes"))
        .filter_async(non_duplicate)
        .endpoint(handle_vote_yes)
}

async fn handle_vote_no(
    bot: AutoSend<Bot>,
    query: CallbackQuery,
    storage: Storage,
) -> HandlerResult {
    let msg = query.message.unwrap();

    if let Some(mut info) = get_from_storage(&storage, msg.chat.id.0, msg.id).await {
        info.vote_count_no += 1;
        info.voters.push(query.from.id.0);

        bot.answer_callback_query(query.id)
            .text("You voted not to delete the message")
            .await?;

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

pub fn vote_no_handler() -> AtomicHandler {
    Update::filter_callback_query()
        .filter(|query: CallbackQuery| query.data.unwrap().eq("vote_no"))
        .filter_async(non_duplicate)
        .endpoint(handle_vote_no)
}

