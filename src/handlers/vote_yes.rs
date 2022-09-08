use teloxide::{
    dispatching::UpdateFilterExt,
    payloads::AnswerCallbackQuerySetters,
    requests::Requester,
    types::{CallbackQuery, Update},
};

use super::{filters::non_duplicate, utils::update_count};
use crate::types::{AtomicHandler, DeleteIttBot, HandlerResult, VoteType};
use crate::Database;

async fn handle_vote_yes(bot: DeleteIttBot, query: CallbackQuery, db: Database) -> HandlerResult {
    if let Some(msg) = query.message {
        if let Ok(Some(mut info)) = db.get_poll(msg.chat.id.0, msg.id).await {
            info.vote_count_yes += 1;

            bot.answer_callback_query(query.id)
                .text("You voted to delete the message")
                .await?;

            if info.vote_count_yes == info.minimum_vote_count {
                bot.delete_message(info.chat_id.to_string(), info.message_id)
                    .await?;

                bot.delete_message(info.chat_id.to_string(), info.poll_id)
                    .await?;

                db.remove_voters(info.id).await?;
                db.remove_poll(info.id).await?;
            } else {
                update_count(&bot, &info).await?;
                db.create_voter(info.id, query.from.id.0.try_into().unwrap())
                    .await?;
                db.register_vote(info.id, VoteType::Yes).await?;
            }
        };
    }

    Ok(())
}

pub fn vote_yes_handler() -> AtomicHandler {
    Update::filter_callback_query()
        .filter(|query: CallbackQuery| query.data.unwrap().eq("vote_yes"))
        .filter_async(non_duplicate)
        .endpoint(handle_vote_yes)
}
