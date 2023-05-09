use log::{debug, trace};
use serde::{Deserialize, Serialize};
use std::net::TcpStream;
use std::thread;

use tungstenite::{stream::MaybeTlsStream, Message, WebSocket};

pub trait Client {
    fn receive(&mut self) -> ServerResponse;
    fn send(&mut self, message: ClientRequest);
}

pub struct WebsocketClient {
    connection: WebSocket<MaybeTlsStream<TcpStream>>,
}

impl WebsocketClient {
    pub fn connect(ws_uri: impl AsRef<str>) -> Self {
        let (connection, _) = tungstenite::connect(ws_uri.as_ref()).unwrap();

        WebsocketClient { connection }
    }
}

impl Client for WebsocketClient {
    fn receive(&mut self) -> ServerResponse {
        let json = loop {
            if let Ok(message) = self.connection.read_message() {
                break message.into_text().unwrap();
            }

            thread::yield_now();
        };

        trace!("received from model: {json}");
        serde_json::from_str(&json).unwrap()
    }

    fn send(&mut self, message: ClientRequest) {
        let json = serde_json::to_string(&message).unwrap();
        debug!("Sending json to client: {json}");
        let ws_message = Message::text(json);
        self.connection.write_message(ws_message).unwrap();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClientRequest {
    StartPredicting { prompt: String },
}

impl ClientRequest {
    fn start_predicting(prompt: String) -> Self {
        ClientRequest::StartPredicting { prompt }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictParameters {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ServerResponse {
    Tokens {
        event: String,
        message_num: usize,
        text: String,
    },
    StreamEnd {
        event: String,
        message_num: usize,
    },
}

impl ServerResponse {
    pub fn text(&self) -> &str {
        match self {
            Self::Tokens { text, .. } => text,
            Self::StreamEnd { .. } => panic!("Expected tokens"),
        }
    }

    pub fn message_num(&self) -> usize {
        match self {
            Self::StreamEnd { message_num, .. } | Self::Tokens { message_num, .. } => *message_num,
        }
    }

    pub fn is_stream_end(&self) -> bool {
        match self {
            Self::Tokens { .. } => false,
            Self::StreamEnd { .. } => true,
        }
    }
}
