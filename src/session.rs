use crate::{
    model_client::{GuidanceRequestBuilder, GuidanceResponse, ModelClient},
    server::{MessageChannel, MessageFromClient, MessageToClient, SessionHandler},
    tools::{web_search::WebSearch, Tool},
};
use async_trait::async_trait;
use futures_util::{StreamExt, TryStreamExt};
use log::{debug, info};
use reqwest_eventsource::Event;
use std::fs;

fn load_prompt_text(prompt_name: &str) -> String {
    let path = format!("src/prompts/{prompt_name}");
    debug!("Reading prompt file: {path}");
    fs::read_to_string(path).expect("Failed to read prompt file")
}

/// A `Session` handles the `Conversation` from beginning to end.
#[derive(Clone)]
pub struct Session<TClient>
where
    TClient: FnOnce() -> Box<dyn ModelClient + Send + Sync> + Send,
{
    make_client: TClient,
}

impl<TClient> Session<TClient>
where
    TClient: FnOnce() -> Box<dyn ModelClient + Send + Sync> + Send,
{
    pub fn new(make_client: TClient) -> Self {
        Self { make_client }
    }
}

#[async_trait]
impl<TClient> SessionHandler for Session<TClient>
where
    TClient: FnOnce() -> Box<dyn ModelClient + Send + Sync> + Send,
{
    async fn handle_session(self, mut channel: impl MessageChannel + Send + Sync) {
        let model_client = (self.make_client)();

        let prompt_preamble = load_prompt_text("guider_preamble.txt");
        let prompt_chat = load_prompt_text("guider_chat.txt");

        let mut history = String::new();

        let mut first_user_input: Option<String> = None;

        loop {
            // Get user's input:
            let user_input: String = {
                let message = channel.receive().await;
                let message: MessageFromClient = serde_json::from_str(&message).unwrap();
                message.message().to_owned()
            };

            if first_user_input.is_none() {
                first_user_input = Some(user_input.clone());
            }

            let first_user_input = first_user_input.as_ref().unwrap();

            // Hack: we need to manually replace {{history}} first, because that value
            // is itself templated, and guidance only performs template replacement once
            let prompt_chat: String = prompt_chat.replace("{{preamble}}", &prompt_preamble);

            let request = GuidanceRequestBuilder::new(&prompt_chat)
                .with_parameter("history", history)
                .with_parameter("user_input", user_input)
                .with_parameter_list("valid_actions", &["WEB_SEARCH", "NONE"])
                .build();

            // Get action / thought response:
            let response = {
                let mut stream = model_client.request_guidance_stream(&request);

                let mut response = GuidanceResponse::new();
                while let Ok(Some(event)) = stream.try_next().await {
                    match event {
                        Event::Open => {}
                        Event::Message(m) => {
                            let delta: GuidanceResponse = serde_json::from_str(&m.data)
                                .expect("response was not in the expected format");

                            response.apply_delta(delta);
                        }
                    }
                }

                response
            };

            // The first response will have THOUGHT and ACTION filled out.
            // let response = model_client.request_guidance(&request).await;
            let action = response.expect_variable("action").trim();
            let action_input = response.expect_variable("action_input").trim();

            // Now we must provide OUTPUT:
            let tool_output = {
                let tool = WebSearch;

                if action == "NONE" {
                    String::new()
                } else {
                    channel
                        .send(MessageToClient::new(
                            String::new(),
                            format!("Searching: {action_input}"),
                            0,
                        ))
                        .await;
                    tool.get_output(action_input, action_input, model_client.as_ref())
                        .await
                }
            };

            // Send OUTPUT back to model and let it continue:
            let response = {
                let ongoing_chat = response.text();
                let request = GuidanceRequestBuilder::new(ongoing_chat)
                    .with_parameter("output", tool_output.clone())
                    .build();

                let mut complete_response = GuidanceResponse::new();
                let mut response_stream = model_client.request_guidance_stream(&request);
                let mut stream_count = 0;
                while let Some(Ok(event)) = response_stream.next().await {
                    match event {
                        Event::Open => info!("stream opened"),
                        Event::Message(m) => {
                            let delta: GuidanceResponse = serde_json::from_str(&m.data)
                                .expect("response was not in the expected format");

                            if let Some(response_delta) = delta.variable("response") {
                                if !response_delta.is_empty() {
                                    let response_delta = if stream_count == 0 {
                                        response_delta.trim_start().to_owned()
                                    } else {
                                        response_delta.to_owned()
                                    };

                                    let to_client = MessageToClient::new(
                                        String::new(),
                                        response_delta,
                                        stream_count,
                                    );
                                    debug!("Sending next part of stream to client: {stream_count}");
                                    channel.send(to_client).await;
                                    stream_count += 1;
                                }
                            }

                            complete_response.apply_delta(delta);
                        }
                    }
                }

                complete_response
            };

            if !tool_output.is_empty() {
                channel
                    .send(MessageToClient::new(
                        String::from("ToolInfo"),
                        tool_output,
                        0,
                    ))
                    .await;
            }

            // Update history
            {
                history = response.text().to_owned();

                let first_user_input_pos = history
                    .find(first_user_input)
                    .expect("expected to see the user's first input in the history");

                // This hack needs to be replaced:
                let preceding_newline_pos = history[0..first_user_input_pos]
                    .rfind('\n')
                    .expect("expected to find a newline before the user's message");

                history = history.split_off(preceding_newline_pos);

                history = history.trim().to_owned();

                // Clear out any "output" sections from history, to save up space in our LLM context
                // major hack:
                history = history
                    .lines()
                    .filter(|l| !l.trim_start().starts_with("[WEB_RESULT"))
                    .collect::<Vec<&str>>()
                    .join("\n");
                history.push('\n');
            }
        }
    }
}
