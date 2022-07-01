use lsp_types::*;
use treecore_lang::message::{Message, MessageHandler};

pub trait RpcChannel {
    /// Send notification to the client.
    fn send_notification(
        &self,
        method: impl Into<String>,
        notification: impl serde::ser::Serialize,
    );

    /// Send request to the client.
    fn send_request(&self, method: impl Into<String>, params: impl serde::ser::Serialize);

    fn window_show_message(&self, typ: MessageType, message: impl Into<String>) {
        self.send_notification(
            "window/showMessage",
            ShowMessageParams {
                typ,
                message: message.into(),
            },
        );
    }

    fn window_log_message(&self, typ: MessageType, message: impl Into<String>) {
        self.send_notification(
            "window/logMessage",
            LogMessageParams {
                typ,
                message: message.into(),
            },
        );
    }

    fn push_msg(&self, msg: Message) {
        if matches!(
            msg.msg_type,
            treecore_lang::message::MessageType::Warning
                | treecore_lang::message::MessageType::Error
        ) {
            self.window_show_message(to_lsp_msg_type(&msg.msg_type), msg.msg.clone());
        }
        self.window_log_message(to_lsp_msg_type(&msg.msg_type), msg.msg);
    }
}

pub struct MessageChannel<'a, T: RpcChannel> {
    channel: &'a T,
}

impl<'a, T: RpcChannel> MessageChannel<'a, T> {
    pub fn new(channel: &'a T) -> MessageChannel<'a, T> {
        MessageChannel { channel }
    }
}

impl<'a, T: RpcChannel> MessageHandler for MessageChannel<'a, T> {
    fn push(&mut self, message: Message) {
        self.channel.push_msg(message);
    }
}

fn to_lsp_msg_type(msg_type: &treecore_lang::message::MessageType) -> MessageType {
    match msg_type {
        treecore_lang::message::MessageType::Error => MessageType::Error,
        treecore_lang::message::MessageType::Warning => MessageType::Warning,
        treecore_lang::message::MessageType::Info => MessageType::Info,
        treecore_lang::message::MessageType::Log => MessageType::Log,
    }
}

#[cfg(test)]
pub mod test_support {

    use pretty_assertions::assert_eq;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::rc::Rc;

    #[derive(Debug)]
    pub enum RpcExpected {
        Notification {
            method: String,
            notification: serde_json::Value,
        },
        /// Check that the string representation of the notification contains a string
        NotificationContainsString { method: String, contains: String },
        Request {
            method: String,
            params: serde_json::Value,
        },
    }

    #[derive(Clone)]
    pub struct RpcMock {
        expected: Rc<RefCell<VecDeque<RpcExpected>>>,
    }

    impl RpcMock {
        pub fn new() -> RpcMock {
            RpcMock {
                expected: Rc::new(RefCell::new(VecDeque::new())),
            }
        }

        pub fn expect_notification(
            &self,
            method: impl Into<String>,
            notification: impl serde::ser::Serialize,
        ) {
            self.expected
                .borrow_mut()
                .push_back(RpcExpected::Notification {
                    method: method.into(),
                    notification: serde_json::to_value(notification).unwrap(),
                });
        }

        fn expect_notification_contains(
            &self,
            method: impl Into<String>,
            contains: impl Into<String>,
        ) {
            self.expected
                .borrow_mut()
                .push_back(RpcExpected::NotificationContainsString {
                    method: method.into(),
                    contains: contains.into(),
                });
        }

        pub fn expect_request(
            &self,
            method: impl Into<String>,
            params: impl serde::ser::Serialize,
        ) {
            self.expected.borrow_mut().push_back(RpcExpected::Request {
                method: method.into(),
                params: serde_json::to_value(params).unwrap(),
            });
        }

        pub fn expect_error_contains(&self, contains: impl Into<String>) {
            let contains = contains.into();
            self.expect_notification_contains("window/showMessage", contains.clone());
            self.expect_notification_contains("window/logMessage", contains);
        }

        pub fn expect_warning_contains(&self, contains: impl Into<String>) {
            self.expect_error_contains(contains)
        }

        pub fn expect_message_contains(&self, contains: impl Into<String>) {
            let contains = contains.into();
            self.expect_notification_contains("window/logMessage", contains);
        }
    }

    impl Drop for RpcMock {
        fn drop(&mut self) {
            if !std::thread::panicking() {
                let expected = self.expected.replace(VecDeque::new());
                if !expected.is_empty() {
                    panic!("Not all expected data was consumed\n{:#?}", expected);
                }
            }
        }
    }

    /// True if any string field of the value has string as a substring
    fn contains_string(value: &serde_json::Value, string: &str) -> bool {
        match value {
            serde_json::Value::Array(values) => {
                values.iter().any(|value| contains_string(value, string))
            }
            serde_json::Value::Object(map) => {
                map.values().any(|value| contains_string(value, string))
            }
            serde_json::Value::String(got_string) => got_string.contains(string),
            serde_json::Value::Null => false,
            serde_json::Value::Bool(..) => false,
            serde_json::Value::Number(..) => false,
        }
    }

    impl super::RpcChannel for RpcMock {
        fn send_notification(
            &self,
            method: impl Into<String>,
            notification: impl serde::ser::Serialize,
        ) {
            let method = method.into();
            let notification = serde_json::to_value(notification).unwrap();
            let expected = self
                .expected
                .borrow_mut()
                .pop_front()
                .ok_or_else(|| {
                    panic!(
                        "No expected value, got notification method={} {:?}",
                        method, notification
                    )
                })
                .unwrap();

            match expected {
                RpcExpected::Notification {
                    method: exp_method,
                    notification: exp_notification,
                } => {
                    assert_eq!((method, notification), (exp_method, exp_notification));
                }
                RpcExpected::NotificationContainsString {
                    method: exp_method,
                    contains,
                } => {
                    assert_eq!(
                        method, exp_method,
                        "{:?} contains {:?}",
                        notification, contains
                    );
                    if !contains_string(&notification, &contains) {
                        panic!(
                            "{:?} does not contain sub-string {:?}",
                            notification, contains
                        );
                    }
                }
                _ => panic!(
                    "Expected {:?}, got notification {} {:?}",
                    expected, method, notification
                ),
            }
        }

        fn send_request(&self, method: impl Into<String>, params: impl serde::ser::Serialize) {
            let method = method.into();
            let params = serde_json::to_value(params).unwrap();
            let expected = self
                .expected
                .borrow_mut()
                .pop_front()
                .ok_or_else(|| {
                    panic!(
                        "No expected value, got request method={} {:?}",
                        method, params
                    )
                })
                .unwrap();

            match expected {
                RpcExpected::Request {
                    method: exp_method,
                    params: exp_params,
                } => {
                    assert_eq!((method, params), (exp_method, exp_params));
                }
                _ => panic!(
                    "Expected {:?}, got request {} {:?}",
                    expected, method, params
                ),
            }
        }
    }
}
