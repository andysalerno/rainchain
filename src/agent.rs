use crate::{conversation::Conversation, model_client::ModelClient, server::MessageChannel};

pub mod action_thought;

pub trait Agent {
    fn handle_assistant_message(
        &self,
        conversation: &mut Conversation,
        channel: &mut dyn MessageChannel,
        client: &dyn ModelClient,
    );
}
