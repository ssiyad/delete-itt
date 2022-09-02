use std::{collections::HashMap, sync::Arc};

use teloxide::{
    dispatching::DpHandlerDescription,
    prelude::{DependencyMap, Handler},
};
use tokio::sync::Mutex;

pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
pub type Storage = Arc<Mutex<HashMap<String, PollInformation>>>;
pub type AtomicHandler = Handler<
    'static,
    DependencyMap,
    Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>,
    DpHandlerDescription,
>;

#[derive(Debug, Clone)]
pub struct PollInformation {
    pub chat_id: i64,
    pub poll_id: i32,
    pub message_id: i32,
    pub minimum_vote_count: u8,
    pub vote_count_yes: u8,
    pub vote_count_no: u8,
    pub voters: Vec<u64>,
}

