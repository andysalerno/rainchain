use crate::{
    client::Client,
    conversation::{self, Conversation},
    server::{MessageChannel, SessionHandler},
};

/// A `Session` handles the `Conversation` from beginning to end.
#[derive(Clone)]
pub struct Session<T>
where
    T: FnOnce() -> Box<dyn Client<String>>,
{
    make_client: T,
}

impl<T> Session<T>
where
    T: FnOnce() -> Box<dyn Client<String>>,
{
    pub fn new(make_client: T) -> Self {
        Self { make_client }
    }
}

impl<T> SessionHandler for Session<T>
where
    T: FnOnce() -> Box<dyn Client<String>>,
{
    fn handle_session(self, channel: impl MessageChannel) {
        let model_client = (self.make_client)();

        let conversation = Conversation::new();
    }
}
