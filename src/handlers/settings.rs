use teloxide::{
    dispatching::{HandlerExt, UpdateFilterExt},
    payloads::SendMessageSetters,
    requests::Requester,
    types::{Message, Update},
    utils::command::BotCommands,
};

use crate::database::Database;
use crate::types::{AtomicHandler, DeleteIttBot, HandlerResult};

#[derive(BotCommands, Clone)]
#[command(rename = "snake_case", description = "some description")]
enum Cmd {
    #[command(description = "Display this text")]
    Help,

    #[command(description = "Set minimum needed votes. Takes an integer as parameter")]
    VoteCount { count: i64 },
}

async fn help_handler(bot: &DeleteIttBot, msg: &Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Cmd::descriptions().to_string())
        .reply_to_message_id(msg.id)
        .await?;

    Ok(())
}

async fn votes_count_handler(
    bot: &DeleteIttBot,
    msg: &Message,
    db: &Database,
    count: i64,
) -> HandlerResult {
    if count <= 0 || count > 10 {
        bot.send_message(msg.chat.id, "Count must be in range of 1 to 10")
            .reply_to_message_id(msg.id)
            .await?;

        return Ok(());
    }

    let chat_id = msg.chat.id.0;

    if let Ok(None) = db.get_chat(chat_id).await {
        db.create_chat(chat_id).await?;
    }

    if let Ok(true) = db.set_chat_votes(chat_id, count).await {
        bot.send_message(msg.chat.id, "Successfully updated minimum vote count")
            .reply_to_message_id(msg.id)
            .await?;
    }

    Ok(())
}

async fn handler(bot: DeleteIttBot, msg: Message, command: Cmd, db: Database) -> HandlerResult {
    match command {
        Cmd::Help => help_handler(&bot, &msg).await,
        Cmd::VoteCount { count } => votes_count_handler(&bot, &msg, &db, count).await,
    }
}

pub fn settings_handler() -> AtomicHandler {
    Update::filter_message()
        .filter_command::<Cmd>()
        .endpoint(handler)
}
