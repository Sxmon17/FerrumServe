use rusqlite::Connection;



pub fn format_message_history(user: &str, messages: &[String]) -> String {
    let mut response = format!("Message history for {}:\n\r", user);
    for message in messages {
        response.push_str(&format!("{}\n\r", message));
    }
    response
}
