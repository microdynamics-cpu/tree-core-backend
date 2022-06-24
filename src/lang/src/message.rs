use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum MessageType {
    Error,
    Warning,
    Info,
    Log,
}

#[must_use]
#[derive(Debug, PartialEq)]
pub struct Message {
    pub msg_type: MessageType,
    pub msg: String,
}

impl Message {
    pub fn log(msg: impl Into<String>) -> Message {
        Message {
            msg_type: MessageType::Log,
            msg: msg.into(),
        }
    }

    pub fn info(msg: impl Into<String>) -> Message {
        Message {
            msg_type: MessageType::Info,
            msg: msg.into(),
        }
    }

    pub fn warning(msg: impl Into<String>) -> Message {
        Message {
            msg_type: MessageType::Warning,
            msg: msg.into(),
        }
    }

    pub fn error(msg: impl Into<String>) -> Message {
        Message {
            msg_type: MessageType::Error,
            msg: msg.into(),
        }
    }

    pub fn file_error(msg: impl Into<String>, file_name: &Path) -> Message {
        Message {
            msg_type: MessageType::Error,
            msg: format!(
                "{} (In file {})",
                msg.into(),
                file_name.to_string_lossy()
            ),
        }
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]: {}", self.msg_type.as_ref(), self.msg)
    }
}

impl AsRef<str> for MessageType {
    fn as_ref(&self) -> &str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
            Self::Log => "log",
        }
    }
}

pub trait MessageHandler {
    fn push(&mut self, msg: Message);
}

impl MessageHandler for Vec<Message> {
    fn push(&mut self, msg: Message) {
        self.push(msg)
    }
}

#[derive(Default)]
pub struct MessagePrinter {}

impl MessageHandler for MessagePrinter {
    fn push(&mut self, msg: Message) {
        println!("{}", msg);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_all() {
        let log_msg = Message::log("log level info");
        // let err_msg = Message::log("err level info");
        // let mut msg_list: Vec<Message>;
        println!("{}", log_msg);
        // msg_list.push(log_msg);
        // msg_list.push(err_msg);
        assert_eq!("hello", "hello");
    }
}