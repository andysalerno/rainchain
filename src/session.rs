use crate::{
    conversation::Conversation,
    model_client::{Client, ClientRequest, PredictParameters, ServerResponse},
    server::{MessageChannel, SessionHandler},
};
use serde::{Deserialize, Serialize};
use serde_json;

/// A `Session` handles the `Conversation` from beginning to end.
#[derive(Clone)]
pub struct Session<T>
where
    T: FnOnce() -> Box<dyn Client>,
{
    make_client: T,
}

impl<T> Session<T>
where
    T: FnOnce() -> Box<dyn Client>,
{
    pub fn new(make_client: T) -> Self {
        Self { make_client }
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

impl<T> SessionHandler for Session<T>
where
    T: FnOnce() -> Box<dyn Client>,
{
    fn handle_session(self, mut channel: impl MessageChannel) {
        let mut model_client = (self.make_client)();

        let _conversation = Conversation::new();

        loop {
            let message = channel.receive();
            let message: MessageFromClient = serde_json::from_str(&message).unwrap();
            println!("message: {message:?}");

            let request = ClientRequest::StartPredicting {
                prompt: message.message,
            };

            model_client.send(request);

            loop {
                println!("getting response...");
                let response = model_client.receive();
                println!("got response: {response:?}");

                channel.send(serde_json::to_string(&response).unwrap());

                if response.is_stream_end() {
                    break;
                }
            }
        }

        println!("Session complete.");
    }
}
