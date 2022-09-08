use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    requests::{Request, Requester},
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message},
};

use crate::database::{Database, Poll};
use crate::types::{DeleteIttBot, HandlerResult};

pub async fn non_duplicate(query: CallbackQuery, db: Database) -> bool {
    match query.message {
        Some(msg) => match db.get_poll(msg.chat.id.0, msg.id).await {
            Ok(Some(poll)) => {
                if let Some(from) = msg.from() {
                    match db.get_voter(poll.id, from.id.0.try_into().unwrap()).await {
                        Ok(o) => o.is_none(),
                        _ => false,
                    }
                } else {
                    false
                }
            }
            _ => false,
        },
        _ => false,
    }
}

pub async fn target_me(bot: DeleteIttBot, msg: Message) -> bool {
    if let Some(t) = msg.text() {
        match bot.get_me().send().await {
            Ok(me) => match me.username.as_ref() {
                Some(username) => {
                    if t.len() == username.len() + 1 {
                        t.starts_with(&format!("@{}", username))
                    } else {
                        t.starts_with(&format!("@{} ", username))
                    }
                }
                _ => false,
            },
            _ => false,
        }
    } else {
        false
    }
}

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
