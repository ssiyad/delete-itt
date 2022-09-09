use std::sync::Arc;

use loon::Dictionary;
use teloxide::{
    adaptors::{cache_me::CacheMe, AutoSend},
    dispatching::DpHandlerDescription,
    prelude::{DependencyMap, Handler},
    Bot,
};

pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
pub type AtomicHandler = Handler<
    'static,
    DependencyMap,
    Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>,
    DpHandlerDescription,
>;
pub type Localization = Arc<Dictionary>;
pub type DeleteIttBot = CacheMe<AutoSend<Bot>>;

pub enum VoteType {
    Yes,
    No,
}
