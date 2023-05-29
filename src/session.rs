use crate::{
    model_client::{GuidanceRequestBuilder, GuidanceResponse, ModelClient},
    server::{MessageChannel, MessageFromClient, MessageToClient, SessionHandler},
    tools::{web_search::WebSearch, Tool},
};
use async_trait::async_trait;
use futures_util::TryStreamExt;
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
                let message = channel.receive();
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
                        Event::Open => info!("got open event"),
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
            info!("Got response: {response:?}");

            // Now we must provide OUTPUT:
            let tool_output = {
                let tool = WebSearch;

                if action == "NONE" {
                    String::new()
                } else {
                    tool.get_output(action_input, action_input, model_client.as_ref())
                        .await
                }
            };

            info!("Got tool output:\n{tool_output}");

            // Send OUTPUT back to model and let it continue:
            let model_response = {
                let ongoing_chat = response.text();
                info!("ongoing chat:\n{:#?}", ongoing_chat);
                let request = GuidanceRequestBuilder::new(ongoing_chat)
                    .with_parameter("output", tool_output)
                    .build();

                let mut response_stream = model_client.request_guidance_stream(&request);
                let mut stream_count = 0;
                while let Ok(Some(event)) = response_stream.try_next().await {
                    match event {
                        Event::Open => info!("got open event"),
                        Event::Message(m) => {
                            let delta: GuidanceResponse = serde_json::from_str(&m.data)
                                .expect("response was not in the expected format");

                            info!("got delta:\n{delta:#?}");

                            if let Some(response_delta) = delta.variable("response") {
                                if !response_delta.is_empty() {
                                    let to_client = MessageToClient::new(
                                        response_delta.to_owned(),
                                        String::new(),
                                        stream_count,
                                    );
                                    channel.send(to_client);
                                    stream_count += 1;
                                }
                            }
                        }
                    }
                }

                let response = model_client.request_guidance(&request).await;
                info!("Got response: {response:?}");
                response
            };
            let ai_response = model_response.expect_variable("response");
            info!("ai response:\n{ai_response}");

            // Send the response to the client.
            {
                let msg_to_client =
                    MessageToClient::new(ai_response.trim().to_owned(), String::new(), 0);

                channel.send(msg_to_client);
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
                info!("New history:\n{history}");
            }
        }
    }
}
