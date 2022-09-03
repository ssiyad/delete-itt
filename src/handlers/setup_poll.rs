use teloxide::{
    adaptors::AutoSend,
    dispatching::UpdateFilterExt,
    payloads::SendMessageSetters,
    requests::Requester,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message, Update},
    Bot,
};

use crate::handlers::utils::target_me;
use crate::storage::put_into_storage;
use crate::types::{AtomicHandler, HandlerResult, PollInformation, Storage};

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

pub fn setup_poll_handler() -> AtomicHandler {
    Update::filter_message()
        .filter_async(target_me)
        .endpoint(setup_poll)
}
