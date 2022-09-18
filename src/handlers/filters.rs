use regex::Regex;
use teloxide::{
    requests::Requester,
    types::{CallbackQuery, Me, Message},
};

use crate::database::Database;
use crate::types::DeleteIttBot;

pub async fn is_privileged(bot: DeleteIttBot, msg: Message) -> bool {
    match msg.from() {
        Some(from) => match bot.get_chat_member(msg.chat.id, from.id).await {
            Ok(member) => member.is_privileged(),
            Err(_) => false,
        },
        None => false,
    }
}

pub fn callback_query_eq<S>(data: S) -> impl Fn(CallbackQuery) -> bool
where
    S: Copy + Into<String>,
{
    move |cq: CallbackQuery| match cq.data {
        Some(d) => d.eq(&data.into()),
        None => false,
    }
}

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

pub async fn target_me(me: Me, msg: Message) -> bool {
    match msg.text() {
        Some(txt) => Regex::new(format!("@{}(\\n|\\s|$)", me.username()).as_str())
            .unwrap()
            .is_match(txt),
        None => false,
    }
}
