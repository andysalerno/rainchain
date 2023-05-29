use crate::{
    agent::{Agent, NextStep},
    conversation::Conversation,
    model_client::{GuidanceRequestBuilder, ModelClient},
    server::{MessageChannel, SessionHandler},
    tools::{web_search::WebSearch, Tool},
};
use async_trait::async_trait;
use log::{debug, info, trace};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write};

fn load_prompt_text(prompt_name: &str) -> String {
    let path = format!("src/prompts/{prompt_name}");
    debug!("Reading prompt file: {path}");
    fs::read_to_string(path).expect("Failed to read prompt file")
}

/// A `Session` handles the `Conversation` from beginning to end.
#[derive(Clone)]
pub struct Session<TClient, TAgent>
where
    TClient: FnOnce() -> Box<dyn ModelClient + Send + Sync> + Send,
    TAgent: FnOnce() -> Box<dyn Agent> + Send,
{
    make_client: TClient,
    make_agent: TAgent,
}

impl<TClient, TAgent> Session<TClient, TAgent>
where
    TClient: FnOnce() -> Box<dyn ModelClient + Send + Sync> + Send,
    TAgent: FnOnce() -> Box<dyn Agent> + Send,
{
    pub fn new(make_client: TClient, make_agent: TAgent) -> Self {
        Self {
            make_client,
            make_agent,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MessageFromClient {
    message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MessageToClient {
    message: String,
}

#[async_trait]
impl<TClient, TAgent> SessionHandler for Session<TClient, TAgent>
where
    TClient: FnOnce() -> Box<dyn ModelClient + Send + Sync> + Send,
    TAgent: FnOnce() -> Box<dyn Agent> + Send,
{
    async fn handle_session(self, _channel: impl MessageChannel + Send + Sync) {
        let model_client = (self.make_client)();

        // let agent = (self.make_agent)();

        let full_history = String::new();
        let prompt_preamble = load_prompt_text("guider_preamble.txt");
        let prompt_chat = load_prompt_text("guider_chat.txt");

        let mut history = String::new();

        let mut first_user_input: Option<String> = None;

        loop {
            // Get user's input:
            let mut user_input = String::new();
            {
                print!("Type your message: ");
                std::io::stdout().flush().unwrap();
                std::io::stdin().read_line(&mut user_input).unwrap();
                user_input = user_input.trim().to_string();
            }

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

            // The first response will have THOUGHT and ACTION filled out.
            let response = model_client.request_guidance(&request).await;
            let action = response.expect_variable("action").trim();
            let action_input = response.expect_variable("action_input").trim();
            info!("Got response: {response:?}");

            // Now we must provide OUTPUT:
            let tool = WebSearch;
            let output = if action == "NONE" {
                String::new()
            } else {
                tool.get_output(action_input, action_input, model_client.as_ref())
                    .await
            };

            info!("Got tool output:\n{output}");

            // Send OUTPUT back to model and let it continue:
            let ongoing_chat = response.text();
            let request = GuidanceRequestBuilder::new(ongoing_chat)
                .with_parameter("output", output)
                .build();
            let response = model_client.request_guidance(&request).await;
            info!("Got response: {response:?}");

            // Update history
            {
                history = response.text().to_owned();

                let first_user_input_pos = history
                    .find(first_user_input)
                    .expect("expected to see the user's first input in the history");

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

        // Outermost conversation loop:
        // 1. get message from user
        // 2. send message to model as prompt, requesting a response
        // 3. start receiving the response until the stream has ended.
        // 4. hand the response to the agent, and allow it to take the next step.
        // 5. Repeat.
        loop {
            // 1. Get a message from a user.
            let message: MessageFromClient = {
                let message = channel.receive();
                serde_json::from_str(&message).unwrap()
            };
        }
    }
}
