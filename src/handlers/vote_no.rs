use loon::Opts;
use teloxide::{
    dispatching::UpdateFilterExt,
    payloads::AnswerCallbackQuerySetters,
    requests::Requester,
    types::{CallbackQuery, Update},
};

use super::{
    filters::{callback_query_eq, non_duplicate},
    utils::get_locale,
    utils::update_count,
};
use crate::types::{AtomicHandler, DeleteIttBot, HandlerResult, Localization, VoteType};
use crate::Database;

async fn handle_vote_no(
    bot: DeleteIttBot,
    query: CallbackQuery,
    db: Database,
    loc: Localization,
) -> HandlerResult {
    if let Some(msg) = query.message {
        if let Ok(Some(mut info)) = db.get_poll(msg.chat.id.0, msg.id).await {
            info.vote_count_no += 1;

            let response = loc.t(
                "vote.voted_to_not_delete",
                Opts::default().locale(&get_locale(&db, msg.chat.id.0).await),
            )?;

            bot.answer_callback_query(query.id).text(response).await?;

            if info.vote_count_no == info.minimum_vote_count {
                bot.delete_message(info.chat_id.to_string(), info.message_id)
                    .await?;

                bot.delete_message(info.chat_id.to_string(), info.poll_id)
                    .await?;

                db.remove_voters(info.id).await?;
                db.remove_poll(info.id).await?;
            } else {
                update_count(&bot, &info, &db, &loc).await?;
                db.create_voter(info.id, query.from.id.0.try_into().unwrap())
                    .await?;
                db.register_vote(info.id, VoteType::No).await?;
            }
        };
    }

    Ok(())
}

pub fn vote_no_handler() -> AtomicHandler {
    Update::filter_callback_query()
        .filter(callback_query_eq("vote_no"))
        .filter_async(non_duplicate)
        .endpoint(handle_vote_no)
}
