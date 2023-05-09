use crate::{conversation::Conversation, server::MessageChannel};

pub mod action_thought;

pub trait Agent {
    fn handle_assistant_message(
        &self,
        conversation: &mut Conversation,
        channel: &mut dyn MessageChannel,
    );
}
