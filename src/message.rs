use chrono::Utc;
use colored::Colorize;

pub struct Message {
    pub sender: String,
    pub content: String,
    pub timestamp: String,
}

impl Message {
    pub fn new(sender: String, content: String) -> Self {
        Message {
            sender,
            content,
            timestamp: Utc::now().format("%H:%M:%S").to_string(),
        }
    }

    pub fn from_database(sender: String, content: String, timestamp: String) -> Self {
        Message {
            sender,
            content,
            timestamp,
        }
    }

    pub fn from_input(sender: String, content: String) -> Self {
        Message {
            sender,
            content,
            timestamp: Utc::now().format("%H:%M:%S").to_string(),
        }
    }

    pub fn format(&self) -> String {
        format!(
            "{} {}: {}",
            self.timestamp.bright_black(),
            self.sender.bright_blue(),
            self.content
        )
    }
}
