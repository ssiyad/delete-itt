pub fn gen_combined_id(chat_id: i64, message_id: i32) -> String {
    format!("{}{}", chat_id, message_id)
}
