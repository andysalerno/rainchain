use std::fs;

use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use log::{debug, info};

use crate::{
    conversation::Conversation,
    model_client::{GuidanceRequestBuilder, GuidanceResponse, ModelClient},
    server::{MessageChannel, MessageToClient},
    tools::{web_search::WebSearch, Tool},
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
        ui_channel: &mut (dyn MessageChannel + Send + Sync),
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

        // The first response will have thought, action, and action_input filled out.
        let action = output.expect_variable("action").trim();
        let action_input = output.expect_variable("action_input").trim();

        // Now we execute the tool selected by the model:
        let tool_output = {
            let tool = WebSearch;

            if action == "NONE" {
                String::new()
            } else {
                ui_channel
                    .send(MessageToClient::new(
                        String::new(),
                        format!("Searching: {action_input}"),
                        0,
                    ))
                    .await;

                tool.get_output(action_input, action_input, self.model_client.as_ref())
                    .await
            }
        };

        let response = {
            let ongoing_chat = output.text();
            let request = GuidanceRequestBuilder::new(ongoing_chat)
                .with_parameter("output", tool_output.clone())
                .build();

            let mut complete_response = GuidanceResponse::new();
            let mut response_stream = self.model_client.request_guidance_stream(&request);
            let mut stream_count = 0;
            // while let Some(Ok(event)) = response_stream.next().await {
            while let Some(delta) = response_stream.next().await {
                if let Some(response_delta) = delta.variable("response") {
                    if !response_delta.is_empty() {
                        let response_delta = if stream_count == 0 {
                            response_delta.trim_start().to_owned()
                        } else {
                            response_delta.to_owned()
                        };

                        let to_client =
                            MessageToClient::new(String::new(), response_delta, stream_count);
                        debug!("Sending next part of stream to client: {stream_count}");
                        ui_channel.send(to_client).await;
                        stream_count += 1;
                    }
                }

                complete_response.apply_delta(delta);
            }

            complete_response
        };

        if !tool_output.is_empty() {
            ui_channel
                .send(MessageToClient::new(
                    String::from("ToolInfo"),
                    tool_output,
                    0,
                ))
                .await;
        }

        // Finally, send the tool output to the model, and let it stream to us the final response:
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
