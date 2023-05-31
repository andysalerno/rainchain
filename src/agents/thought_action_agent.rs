use async_trait::async_trait;
use futures_util::{Stream, StreamExt};

use crate::{
    conversation::Conversation,
    model_client::{GuidanceRequestBuilder, ModelClient},
};

use super::Agent;

pub struct ThoughtActionAgent {
    model_client: Box<dyn ModelClient + Send + Sync>,
    full_history: Conversation,
}

impl ThoughtActionAgent {
    pub fn new(model_client: Box<dyn ModelClient + Send + Sync>) -> Self {
        Self {
            model_client,
            full_history: Conversation::new(),
        }
    }
}

#[async_trait]
impl Agent for ThoughtActionAgent {
    async fn get_response(&mut self, _message: &str) -> String {
        todo!()
    }

    async fn get_response_stream(
        &mut self,
        message: &str,
    ) -> Box<dyn Stream<Item = String> + Unpin + Send> {
        self.full_history.add_user_message(message);

        let request = GuidanceRequestBuilder::new("some text").build();
        let stream = self.model_client.request_guidance_stream(&request);

        Box::new(stream.map(|s| s.text().to_owned()))
    }
}
