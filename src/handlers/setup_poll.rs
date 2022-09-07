use teloxide::{
    dispatching::UpdateFilterExt,
    payloads::SendMessageSetters,
    requests::Requester,
    types::{Message, Update},
};

use super::utils::{target_me, update_count};

use crate::types::{AtomicHandler, DeleteIttBot, HandlerResult};
use crate::Database;

async fn setup_poll(bot: DeleteIttBot, msg: Message, db: Database) -> HandlerResult {
    if let Some(reply_to_message_id) = msg.reply_to_message() {
        bot.delete_message(msg.chat.id, msg.id).await?;

        let poll_msg = bot
            .send_message(
                msg.chat.id,
                "Should I delete this message? Minimum number of vote needed is 5",
            )
            .reply_to_message_id(reply_to_message_id.id)
            .await?;

        db.create_poll(msg.chat.id.0, poll_msg.id, reply_to_message_id.id, 5)
            .await
            .unwrap();

        if let Ok(Some(e)) = db.get_poll(msg.chat.id.0, poll_msg.id).await {
            update_count(&bot, &e).await?;
        }
    }

    Ok(())
}

pub fn setup_poll_handler() -> AtomicHandler {
    Update::filter_message()
        .filter_async(target_me)
        .endpoint(setup_poll)
}
