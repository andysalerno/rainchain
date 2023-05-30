use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpListener, TcpStream},
};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{accept, Message, WebSocket},
    WebSocketStream,
};

// use tungstenite::{accept, Message, WebSocket};
// use tokio_tungstenite::{accept_async, Message}

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

#[async_trait]
pub trait MessageChannel {
    // fn send(&mut self, message: String);
    async fn send(&mut self, message: MessageToClient);
    async fn receive(&mut self) -> String;
}

#[async_trait]
impl<S> MessageChannel for WebSocketStream<S>
where
    S: AsyncRead + AsyncWrite + Send + Unpin,
{
    async fn send(&mut self, message: MessageToClient) {
        let json = serde_json::to_string(&message).expect("Could not serialize message to json");
        futures::SinkExt::send(self, Message::Text("blah".into())).await;
    }

    async fn receive(&mut self) -> String {
        // TODO: unwrap unwrap? :(
        let message = self.next().await.unwrap().unwrap();

        message.into_text().unwrap()
    }
}

struct WsChannel {}

pub struct WebsocketServer {}

#[async_trait]
impl Server for WebsocketServer {
    async fn run<T>(self, session_handler: T)
    where
        T: SessionHandler + Clone + Send + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:5007").await.unwrap();

        while let Ok((stream, _)) = listener.accept().await {
            println!("new incoming stream.");

            let session_handler = session_handler.clone();

            tokio::task::spawn(async move {
                let websocket = accept_async(stream).await.unwrap();

                let session_handler = session_handler;
                session_handler.handle_session(websocket).await;
            });
        }
    }
}
