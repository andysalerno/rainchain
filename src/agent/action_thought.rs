use log::{debug};

use crate::{conversation::Conversation, server::MessageChannel};

use super::Agent;

pub struct ActionThought {}

impl ActionThought {
    pub fn new() -> Self {
        Self {}
    }
}

impl Agent for ActionThought {
    fn handle_assistant_message(
        &self,
        _conversation: &mut Conversation,
        _channel: &mut dyn MessageChannel,
    ) {
        debug!("ActionThought agent saw message from assistant.");
    }
}
