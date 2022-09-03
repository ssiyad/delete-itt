use teloxide::{
    adaptors::AutoSend,
    requests::Requester,
    types::{CallbackQuery, Message},
    Bot,
};

use crate::storage::get_from_storage;
use crate::types::Storage;

pub async fn non_duplicate(query: CallbackQuery, storage: Storage) -> bool {
    let msg = query.message.unwrap();

    if let Some(info) = get_from_storage(&storage, msg.chat.id.0, msg.id).await {
        !info.voters.contains(&query.from.id.0)
    } else {
        false
    }
}

pub async fn target_me(bot: AutoSend<Bot>, msg: Message) -> bool {
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
