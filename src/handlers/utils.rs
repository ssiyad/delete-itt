use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    requests::{Request, Requester},
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message},
};

use crate::storage::get_from_storage;
use crate::types::{DeleteIttBot, HandlerResult, PollInformation, Storage};

pub async fn non_duplicate(query: CallbackQuery, storage: Storage) -> bool {
    let msg = query.message.unwrap();

    if let Some(info) = get_from_storage(&storage, msg.chat.id.0, msg.id).await {
        !info.voters.contains(&query.from.id.0)
    } else {
        false
    }
}

pub async fn target_me(bot: DeleteIttBot, msg: Message) -> bool {
    if let Some(t) = msg.text() {
        let me = bot.get_me().send().await.unwrap();
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

fn format_vote_button(text: &str, count: u8) -> String {
    format!("{} ({})", text, count)
}

pub fn gen_markup(yes_count: u8, no_count: u8) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::default().append_row(vec![
        InlineKeyboardButton::callback(format_vote_button("Yes", yes_count), "vote_yes"),
        InlineKeyboardButton::callback(format_vote_button("No", no_count), "vote_no"),
    ])
}

pub async fn update_count(bot: &DeleteIttBot, info: &PollInformation) -> HandlerResult {
    bot.edit_message_reply_markup(info.chat_id.to_string(), info.poll_id)
        .reply_markup(gen_markup(info.vote_count_yes, info.vote_count_no))
        .await?;

    Ok(())
}
