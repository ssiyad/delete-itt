use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    requests::Requester,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};

use crate::database::{Database, Poll};
use crate::types::{DeleteIttBot, HandlerResult};

fn format_vote_button(text: &str, count: i64) -> String {
    format!("{} ({})", text, count)
}

pub fn gen_markup(yes_count: i64, no_count: i64) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::default().append_row(vec![
        InlineKeyboardButton::callback(format_vote_button("Yes", yes_count), "vote_yes"),
        InlineKeyboardButton::callback(format_vote_button("No", no_count), "vote_no"),
    ])
}

pub async fn update_count(bot: &DeleteIttBot, info: &Poll) -> HandlerResult {
    bot.edit_message_reply_markup(info.chat_id.to_string(), info.poll_id)
        .reply_markup(gen_markup(info.vote_count_yes, info.vote_count_no))
        .await?;

    Ok(())
}

pub async fn get_locale(db: &Database, chat_id: i64) -> String {
    match db.get_chat_locale(chat_id).await {
        Ok(Some(lang)) => lang,
        _ => "en".to_string(),
    }
}
