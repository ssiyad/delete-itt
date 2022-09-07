use teloxide::{
    dispatching::{HandlerExt, UpdateFilterExt},
    payloads::SendMessageSetters,
    requests::Requester,
    types::{Message, Update},
    utils::command::BotCommands,
};

use crate::types::{AtomicHandler, DeleteIttBot, HandlerResult};

#[derive(BotCommands, Clone)]
#[command(rename = "snake_case", description = "some description")]
enum Cmd {
    #[command(description = "Display this text")]
    Help,
}

async fn help_handler(bot: &DeleteIttBot, msg: &Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Cmd::descriptions().to_string())
        .reply_to_message_id(msg.id)
        .await?;

    Ok(())
}

async fn handler(bot: DeleteIttBot, msg: Message, command: Cmd) -> HandlerResult {
    match command {
        Cmd::Help => help_handler(&bot, &msg).await,
    }
}

pub fn settings_handler() -> AtomicHandler {
    Update::filter_message()
        .filter_command::<Cmd>()
        .endpoint(handler)
}
