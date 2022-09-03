use teloxide::{
    dispatching::UpdateFilterExt,
    payloads::AnswerCallbackQuerySetters,
    requests::Requester,
    types::{CallbackQuery, Update},
};

use crate::handlers::utils::{non_duplicate, update_count};
use crate::storage::{get_from_storage, put_into_storage, remove_from_storage};
use crate::types::{AtomicHandler, DeleteIttBot, HandlerResult, Storage};

async fn handle_vote_yes(
    bot: DeleteIttBot,
    query: CallbackQuery,
    storage: Storage,
) -> HandlerResult {
    let msg = query.message.unwrap();

    if let Some(mut info) = get_from_storage(&storage, msg.chat.id.0, msg.id).await {
        info.vote_count_yes += 1;
        info.voters.push(query.from.id.0);

        bot.answer_callback_query(query.id)
            .text("You voted to delete the message")
            .await?;

        update_count(&bot, &info).await?;

        if info.vote_count_yes == info.minimum_vote_count {
            bot.delete_message(info.chat_id.to_string(), info.message_id)
                .await
                .unwrap();

            bot.delete_message(info.chat_id.to_string(), info.poll_id)
                .await?;

            remove_from_storage(&storage, msg.chat.id.0, msg.id).await;
        } else {
            put_into_storage(&storage, msg.chat.id.0, msg.id, info).await;
        }
    };

    Ok(())
}

pub fn vote_yes_handler() -> AtomicHandler {
    Update::filter_callback_query()
        .filter(|query: CallbackQuery| query.data.unwrap().eq("vote_yes"))
        .filter_async(non_duplicate)
        .endpoint(handle_vote_yes)
}
