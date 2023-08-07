use std::collections::HashMap;

use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use log::{info, warn};

use crate::{
    conversation::{ChatMessage, Conversation},
    load_prompt_text,
    model_client::{
        GuidanceRequestBuilder, GuidanceResponse, MemoryGetRequest, MemoryStoreRequest, ModelClient,
    },
    server::{MessageChannel, MessageToClient},
    tools::{web_search::WebSearch, Tool},
};

use super::{
    intent_detector::{Intent, IntentDetector},
    Agent,
};

pub struct ThoughtActionAgent {
    model_client: Box<dyn ModelClient + Send + Sync>,
    conversation: Conversation,
}

impl ThoughtActionAgent {
    pub fn new(model_client: Box<dyn ModelClient + Send + Sync>) -> Self {
        Self {
            model_client,
            conversation: Conversation::new(),
        }
    }

    fn intent_detector() -> IntentDetector {
        let prompt = load_prompt_text("intent_detection.txt");
        IntentDetector::new(
            vec![
                Intent::new("information_retrieval", "The user intends to retrieve information from some knowledge store, such as the web."),
            ],
            prompt,
        )
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
    ) -> Box<dyn Stream<Item = Option<String>> + Unpin + Send> {
        // let prompt_preamble = load_prompt_text("guider_preamble.txt");
        warn!("Loading llama2chat preamble");
        let prompt_preamble = load_prompt_text("guider_preamble_llama2chat.txt");
        let prompt_chat = load_prompt_text("guider_chat.txt");

        self.conversation
            .add_message(ChatMessage::User(message.into()));

        // Hack: we need to manually replace {{history}} first, because that value
        // is itself templated, and guidance only performs template replacement once
        let prompt_chat: String = prompt_chat.replace("{{preamble}}", &prompt_preamble);

        let history = self.conversation.build_history();
        let prompt_chat: String = prompt_chat.replace("{{history~}}", &history);

        info!("Build prompt_chat:\n{prompt_chat}");

        // First, as the ThoughtActionAgent, we get the thought/action output:
        let request = GuidanceRequestBuilder::new(prompt_chat)
            .with_parameter("user_input", message)
            .with_parameter_list("valid_actions", &["WEB_SEARCH", "NONE"])
            .build();

        let output = self.model_client.request_guidance(&request).await;

        info!("Got thought/action output:\n{:#?}", output);

        // Testing memory and intent detection:
        {
            // let memory_request = MemoryGetRequest {
            //     query: "hello".to_owned(),
            // };
            // let memory_response = self.model_client.request_memory(&memory_request).await;
            // let intent_detector = Self::intent_detector();
            // let intent = intent_detector
            //     .detect_intent(self.model_client.as_ref(), &self.conversation)
            //     .await;

            // info!("Detected intent: {intent}");
        }

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

            while let Some(Some(delta)) = response_stream.next().await {
                if let Some(response_delta) = delta.variable("response") {
                    if !response_delta.is_empty() {
                        let response_delta = if stream_count == 0 {
                            response_delta.trim_start().to_owned()
                        } else {
                            response_delta.to_owned()
                        };

                        let to_client =
                            MessageToClient::new(String::new(), response_delta, stream_count);

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

        let response_text = response.expect_variable("response");
        info!(
            "\n\n-----------------\nReponse is\n{:?}\n------------------\n\n",
            response_text
        );

        let assistant_message = build_assistant_chat_message(action, action_input, response_text);
        info!("Added assistant message:\n{:?}", assistant_message);
        self.conversation.add_message(assistant_message);

        // Store user and assistant output for just this turn as a document
        {
            let last_two_range = self.conversation.messages().len() - 2..;
            let last_two_messages = &self.conversation.messages()[last_two_range];

            let messages_stringified = Conversation::messages_to_string(last_two_messages);

            let mut memory_request = MemoryStoreRequest::new();

            // Empty ID value is ok, server will generate a guid for it:
            memory_request.add_document("", messages_stringified, HashMap::new());

            self.model_client.store_memory(&memory_request).await;
        }

        // We will return nothing, since we already sent the client everything ourselves. No need to make the session do it for us.
        Box::new(futures::stream::empty())
    }
}

fn build_assistant_chat_message(action: &str, action_input: &str, response: &str) -> ChatMessage {
    let mut template = load_prompt_text("thought_action_response.txt");
    template = template.replace("{{action}}", action.trim());
    template = template.replace("{{action_input}}", action_input.trim());
    template = template.replace("{{response}}", response.trim());
    ChatMessage::Assistant(template)
}
