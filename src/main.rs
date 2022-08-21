use std::sync::Arc;

use tokio::sync::Mutex;
use teloxide::{prelude::*, dispatching::UpdateHandler};

#[derive(Debug, Clone)]
struct VoterInformation {
    user_id: i32,
    vote_option: Vec<i32>,
}

#[derive(Debug, Clone)]
struct PollInformation {
    chat_id: i64,
    poll_id: i32,
    message_id: i32,
    minimum_vote_count: u8,
    vote_count: Vec<u8>,
    voters: Vec<VoterInformation>,
}

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

async fn add_to_storage(storage: Arc<Mutex<Vec<PollInformation>>>, data: PollInformation) {
    let mut s = storage.lock().await;
    s.push(data);
}

async fn get_from_storage(storage: Arc<Mutex<Vec<PollInformation>>>, chat_id: i64, message_id: i32) -> Option<PollInformation> {
    let s = storage.lock().await;

    for i in s.clone().into_iter() {
        if i.chat_id == chat_id && i.message_id == message_id {
            return Some(i);
        }
    };

    None
}

async fn setup_poll(bot: AutoSend<Bot>, msg: Message, storage: Arc<Mutex<Vec<PollInformation>>>) -> HandlerResult {
    dbg!(&storage);

    if let Some(reply_to_message_id) = msg.reply_to_message() {
        if (get_from_storage(storage.clone(), msg.chat.id.0, reply_to_message_id.id).await).is_some() {
            // TODO: respond with current_poll
            return Ok(());
        }

        bot
            .delete_message(msg.chat.id, msg.id)
            .await?;

        let poll = bot
            .send_poll(
                msg.chat.id,
                "Should I delete this message?", 
                [
                    String::from("Yes"),
                    String::from("No"),
                ],
            )
            .is_anonymous(false)
            .reply_to_message_id(reply_to_message_id.id)
            .await?;

        let info = PollInformation {
            chat_id: msg.chat.id.0,
            poll_id: poll.id,
            message_id: reply_to_message_id.id,
            minimum_vote_count: 3,
            vote_count: vec![],
            voters: vec![]
        };

        add_to_storage(storage, info).await;
    }
    
    Ok(())
}

async fn poll_answer_handler(bot: AutoSend<Bot>, poll: PollAnswer) -> HandlerResult {
    todo!()
}

async fn fun_log(bot: AutoSend<Bot>) -> HandlerResult {
    todo!()
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    let message_handler = Update::filter_message().endpoint(setup_poll);
    let poll_answer_handler = Update::filter_poll_answer().endpoint(poll_answer_handler);

    teloxide::dptree::entry()
        .branch(message_handler)
        .branch(poll_answer_handler)
}

#[tokio::main]
async fn main() {
    let bot = Bot::from_env().auto_send();
    let storage: Arc<Mutex<Vec<PollInformation>>> = Arc::new(Mutex::new(vec![]));

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![storage])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

