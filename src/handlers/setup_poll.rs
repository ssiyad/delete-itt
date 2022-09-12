use loon::Opts;
use teloxide::{
    dispatching::UpdateFilterExt,
    payloads::SendMessageSetters,
    requests::Requester,
    types::{Message, Update},
};

use super::filters::target_me;
use super::utils::{get_locale, update_count};

use crate::types::{AtomicHandler, DeleteIttBot, HandlerResult, Localization};
use crate::Database;

async fn setup_poll(
    bot: DeleteIttBot,
    msg: Message,
    db: Database,
    loc: Localization,
) -> HandlerResult {
    if let Some(reply_to_message_id) = msg.reply_to_message() {
        bot.delete_message(msg.chat.id, msg.id).await?;

        let min_vote_count = db
            .get_chat_votes(msg.chat.id.0)
            .await
            .unwrap_or(Some(5))
            .unwrap_or(5);

        let response = loc.t(
            "vote.title",
            Opts::default()
                .var("count", min_vote_count)
                .locale(&get_locale(&db, msg.chat.id.0).await),
        )?;

        let poll_msg = bot
            .send_message(msg.chat.id, response)
            .reply_to_message_id(reply_to_message_id.id)
            .await?;

        db.create_poll(
            msg.chat.id.0,
            poll_msg.id,
            reply_to_message_id.id,
            min_vote_count,
        )
        .await?;

        if let Ok(Some(e)) = db.get_poll(msg.chat.id.0, poll_msg.id).await {
            update_count(&bot, &e, &db, &loc).await?;
        }
    }

    Ok(())
}

pub fn setup_poll_handler() -> AtomicHandler {
    Update::filter_message()
        .filter_async(target_me)
        .endpoint(setup_poll)
}
