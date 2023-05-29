use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::net::TcpListener;

use tungstenite::{accept, Message, WebSocket};

#[async_trait]
pub trait Server {
    async fn run<T>(self, session_handler: T)
    where
        T: SessionHandler + Clone + Send + 'static;
}

#[async_trait]
pub trait SessionHandler {
    async fn handle_session(self, channel: impl MessageChannel + Send + Sync);
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageFromClient {
    message: String,
}

impl MessageFromClient {
    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageToClient {
    event: String,
    message_num: usize,
    text: String,
}

impl MessageToClient {
    pub fn new(event: String, text: String, message_num: usize) -> Self {
        Self {
            event,
            message_num,
            text,
        }
    }
}

pub trait MessageChannel {
    // fn send(&mut self, message: String);
    fn send(&mut self, message: MessageToClient);
    fn receive(&mut self) -> String;
}

impl<Stream> MessageChannel for WebSocket<Stream>
where
    Stream: std::io::Read + std::io::Write,
{
    fn send(&mut self, message: MessageToClient) {
        let json = serde_json::to_string(&message).expect("Could not serialize message to json");
        self.write_message(Message::Text(json)).unwrap();
    }

    fn receive(&mut self) -> String {
        let message = self.read_message().unwrap();

        message.into_text().unwrap()
    }
}

pub struct WebsocketServer {}

#[async_trait]
impl Server for WebsocketServer {
    async fn run<T>(self, session_handler: T)
    where
        T: SessionHandler + Clone + Send + 'static,
    {
        let server = TcpListener::bind("127.0.0.1:5007").unwrap();

        for stream in server.incoming() {
            println!("new incoming stream.");

            let session_handler = session_handler.clone();

            tokio::task::spawn(async move {
                let websocket = accept(stream.unwrap()).unwrap();

                let session_handler = session_handler;
                session_handler.handle_session(websocket).await;
            });
        }
    }
}
