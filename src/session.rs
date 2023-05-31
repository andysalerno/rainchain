use async_trait::async_trait;
use futures_util::StreamExt;
use log::{debug, info};

use crate::{
    agents::Agent,
    server::{MessageChannel, MessageFromClient, MessageToClient, SessionHandler},
};

#[derive(Clone)]
pub struct AgentSessionHandler<TAgent>
where
    TAgent: FnOnce() -> Box<dyn Agent + Send + Sync> + Send,
{
    make_agent: TAgent,
}

impl<TAgent> AgentSessionHandler<TAgent>
where
    TAgent: FnOnce() -> Box<dyn Agent + Send + Sync> + Send,
{
    pub fn new(make_agent: TAgent) -> Self {
        Self { make_agent }
    }
}

#[async_trait]
impl<TAgent> SessionHandler for AgentSessionHandler<TAgent>
where
    TAgent: FnOnce() -> Box<dyn Agent + Send + Sync> + Send,
{
    async fn handle_session(self, mut ui_channel: impl MessageChannel + Send + Sync) {
        info!("Session opened");

        let mut agent = (self.make_agent)();

        loop {
            // Get user's input:
            info!("Waiting for input from user...");
            let user_input: String = {
                let message = ui_channel.receive().await;
                let message: MessageFromClient = serde_json::from_str(&message).unwrap();
                message.message().to_owned()
            };
            info!("Got input from user: {user_input}");

            // You can get a full response:
            // let agent_response = agent.get_response(&user_input).await;

            // But we will stream the response piece by piece:
            info!("Requesting response from agent...");
            let mut response = String::new();
            let mut stream = agent
                .get_response_stream(&user_input, &mut ui_channel)
                .await;

            // Yes, it is an Option<Option<T>>
            // Outer layer is from the Stream, inner layer is our own data.
            // A None from either indicates we should stop.
            while let Some(Some(t)) = stream.next().await {
                debug!("Received input from agent stream: {t}");
                response.push_str(&t);
            }
            info!("Finished reading response from agent stream:\n{response}");

            // Re-bind, because we no longer need it to be mut:
            let response = response;

            ui_channel
                .send(MessageToClient::new(String::new(), response, 0))
                .await;
        }
    }
}

// Server: spawns Sessions
// Session: loops over a Conversation; uses Agent to build response to user; has a ClientChannel to send back to user
// ActionThoughtAgent: impl Agent. Has a GuidanceClient for talking to GuidanceServer
// let agent = Agent();
// let server = Server(|| SessionHandler::new())
// server.run()
