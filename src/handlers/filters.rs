use teloxide::{
    requests::{Request, Requester},
    types::{CallbackQuery, Message},
};

use crate::database::Database;
use crate::types::DeleteIttBot;

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
