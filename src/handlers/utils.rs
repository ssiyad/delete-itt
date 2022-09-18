use loon::Opts;
use teloxide::{
    payloads::EditMessageReplyMarkupSetters,
    requests::Requester,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
};

use crate::database::{Database, Poll};
use crate::types::{DeleteIttBot, HandlerResult, Localization};

fn format_vote_button(text: &str, count: i64) -> String {
    format!("{} ({})", text, count)
}

pub fn gen_markup(
    yes_count: i64,
    no_count: i64,
    yes_content: &str,
    no_content: &str,
) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::default().append_row(vec![
        InlineKeyboardButton::callback(format_vote_button(yes_content, yes_count), "vote_yes"),
        InlineKeyboardButton::callback(format_vote_button(no_content, no_count), "vote_no"),
    ])
}

pub async fn update_count(
    bot: &DeleteIttBot,
    info: &Poll,
    db: &Database,
    loc: &Localization,
) -> HandlerResult {
    let locale = &get_locale(db, info.chat_id).await;

    let yes_txt = loc.t("vote.yes", Opts::default().locale(locale))?;

    let no_txt = loc.t("vote.no", Opts::default().locale(locale))?;

    bot.edit_message_reply_markup(info.chat_id.to_string(), info.poll_id)
        .reply_markup(gen_markup(
            info.vote_count_yes,
            info.vote_count_no,
            &yes_txt,
            &no_txt,
        ))
        .await?;

    Ok(())
}

pub async fn get_locale(db: &Database, chat_id: i64) -> String {
    match db.get_chat_locale(chat_id).await {
        Ok(Some(lang)) => lang,
        _ => "en".to_string(),
    }
}

pub async fn get_poll_delete_delay(db: &Database, chat_id: i64) -> i64 {
    match db.get_chat_poll_delete_delay(chat_id).await {
        Ok(Some(delay)) => delay,
        _ => 5,
    }
}

pub async fn delete_message(bot: DeleteIttBot, msg: Message) -> HandlerResult {
    bot.delete_message(msg.chat.id, msg.id).await?;

    Ok(())
}
