use crate::{conversation::Conversation, model_client::ModelClient, server::MessageChannel};

pub mod action_thought;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NextStep {
    KeepPredicting,
    StopPredicting,
}

pub trait Agent {
    fn handle_assistant_message(
        &self,
        conversation: &mut Conversation,
        channel: &mut dyn MessageChannel,
        client: &mut dyn ModelClient,
    ) -> NextStep;

    fn bot_message_prefix(&self) -> String;
}
