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
pub type DeleteIttBot = CacheMe<AutoSend<Bot>>;

pub enum VoteType {
    Yes,
    No,
}
