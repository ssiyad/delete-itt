use loon::Opts;
use teloxide::{
    dispatching::UpdateFilterExt,
    payloads::AnswerCallbackQuerySetters,
    requests::Requester,
    types::{CallbackQuery, Update},
};

use super::{
    filters::{callback_query_eq, non_duplicate},
    utils::{get_locale, update_count},
};
use crate::types::{AtomicHandler, DeleteIttBot, HandlerResult, Localization, VoteType};
use crate::Database;

async fn handle_vote_yes(
    bot: DeleteIttBot,
    query: CallbackQuery,
    db: Database,
    loc: Localization,
) -> HandlerResult {
    if let Some(msg) = query.message {
        if let Ok(Some(mut info)) = db.get_poll(msg.chat.id.0, msg.id).await {
            info.vote_count_yes += 1;

            let response = loc.t(
                "vote.voted_to_delete",
                Opts::default().locale(&get_locale(&db, msg.chat.id.0).await),
            )?;

            bot.answer_callback_query(query.id).text(response).await?;

            if info.vote_count_yes == info.minimum_vote_count {
                bot.delete_message(info.chat_id.to_string(), info.message_id)
                    .await?;

                bot.edit_message_reply_markup(info.chat_id.to_string(), info.poll_id)
                    .await
                    .unwrap();

                db.remove_voters(info.id).await?;
                db.remove_poll(info.id).await?;
                db.schedule_message_delete(
                    info.chat_id,
                    info.poll_id.into(),
                    (std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        + std::time::Duration::from_secs(5))
                    .as_secs()
                    .try_into()
                    .unwrap(),
                )
                .await?;
            } else {
                update_count(&bot, &info, &db, &loc).await?;
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
        .filter(callback_query_eq("vote_yes"))
        .filter_async(non_duplicate)
        .endpoint(handle_vote_yes)
}
