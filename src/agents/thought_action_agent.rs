use std::fs;

use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use log::{debug, info};

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
        let prompt_preamble = load_prompt_text("guider_preamble.txt");
        let prompt_chat = load_prompt_text("guider_chat.txt");

        // Hack: we need to manually replace {{history}} first, because that value
        // is itself templated, and guidance only performs template replacement once
        let prompt_chat: String = prompt_chat.replace("{{preamble}}", &prompt_preamble);

        // First, as the ThoughtActionAgent, we get the thought/action output:
        let request = GuidanceRequestBuilder::new(prompt_chat)
            // .with_parameter("history", conversation.full_history())
            .with_parameter("history", "")
            .with_parameter("user_input", message)
            .with_parameter_list("valid_actions", &["WEB_SEARCH", "NONE"])
            .build();

        let output = self.model_client.request_guidance(&request).await;

        info!("Got thought/action output:\n{:#?}", output);

        todo!();

        // Next, execute the tool, then get the response
        let stream = self.model_client.request_guidance_stream(&request);

        self.full_history.add_user_message(message);

        Box::new(stream.map(|s| s.text().to_owned()))
    }
}

fn load_prompt_text(prompt_name: &str) -> String {
    let path = format!("src/prompts/{prompt_name}");
    debug!("Reading prompt file: {path}");
    fs::read_to_string(path).expect("Failed to read prompt file")
}
