use crate::{
    agent::Agent,
    conversation::Conversation,
    model_client::{ClientRequest, ModelClient},
    server::{MessageChannel, SessionHandler},
};
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use std::fs;

fn load_prompt_text(prompt_name: &str) -> String {
    let path = format!("src/prompts/{prompt_name}");
    debug!("Reading prompt file: {path}");
    fs::read_to_string(path).expect("Failed to read prompt file")
}

/// A `Session` handles the `Conversation` from beginning to end.
#[derive(Clone)]
pub struct Session<TClient, TAgent>
where
    TClient: FnOnce() -> Box<dyn ModelClient>,
    TAgent: FnOnce() -> Box<dyn Agent>,
{
    make_client: TClient,
    make_agent: TAgent,
}

impl<TClient, TAgent> Session<TClient, TAgent>
where
    TClient: FnOnce() -> Box<dyn ModelClient>,
    TAgent: FnOnce() -> Box<dyn Agent>,
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

impl<TClient, TAgent> SessionHandler for Session<TClient, TAgent>
where
    TClient: FnOnce() -> Box<dyn ModelClient>,
    TAgent: FnOnce() -> Box<dyn Agent>,
{
    fn handle_session(self, mut channel: impl MessageChannel) {
        let mut model_client = (self.make_client)();

        let mut conversation = {
            let context = load_prompt_text("wizardvicuna_actionthought.txt");
            let context = context.trim();
            Conversation::new(context)
        };

        let agent = (self.make_agent)();

        // Outermost conversation loop:
        // 1. get message from user
        // 2. send message to model as prompt, requesting a response
        // 3. start receiving the response until the stream has ended.
        // 4. hand the response to the agent, and allow it to take the next step.
        // 5. Repeat.
        loop {
            // 1. Get a message from a user.
            let message = channel.receive();
            let message: MessageFromClient = serde_json::from_str(&message).unwrap();

            conversation.add_user_message(message.message.clone());

            // Add an empty message from the assistant - this will be completed by the model.
            conversation.add_assistant_message(String::new());

            // 2. Tell the model to start predicting
            {
                let request = ClientRequest::start_predicting(conversation.build_full_history());

                model_client.send(request);
            }

            // 3. Start receiving text from the model.
            debug!("Starting streaming from model...");
            loop {
                let response = model_client.receive();
                trace!("{response:?}");

                if response.message_num() == 0 {
                    conversation.add_assistant_message(response.text().into());
                } else if !response.is_stream_end() {
                    conversation.append_to_last_assistant_message(response.text());
                }

                channel.send(serde_json::to_string(&response).unwrap());

                if response.is_stream_end() {
                    debug!("...done (encountered stream end)");
                    debug!("{}", conversation.last_assistant_message());

                    // 4. hand off the message to the agent, so it can decide what to do next.
                    agent.handle_assistant_message(
                        &mut conversation,
                        &mut channel,
                        model_client.as_ref(),
                    );
                    break;
                }
            }
        }
    }
}
