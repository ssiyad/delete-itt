use crate::types::{PollInformation, Storage};
use crate::utils::gen_combined_id;

pub async fn get_from_storage(
    storage: &Storage,
    chat_id: i64,
    message_id: i32,
) -> Option<PollInformation> {
    let s = storage.lock().await;
    s.get(&gen_combined_id(chat_id, message_id)).cloned()
}

pub async fn put_into_storage(
    storage: &Storage,
    chat_id: i64,
    message_id: i32,
    data: PollInformation,
) -> Option<PollInformation> {
    let mut s = storage.lock().await;
    s.insert(gen_combined_id(chat_id, message_id), data)
}

pub async fn remove_from_storage(
    storage: &Storage,
    chat_id: i64,
    message_id: i32,
) -> Option<PollInformation> {
    let mut s = storage.lock().await;
    s.remove(&gen_combined_id(chat_id, message_id))
}
