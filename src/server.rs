use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::{accept, Message, WebSocket};

pub trait Server {
    fn run<T>(self, session_handler: T)
    where
        T: SessionHandler + Clone + Send + 'static;
}

pub trait SessionHandler {
    fn handle_session(self, channel: impl MessageChannel);
}

pub trait MessageChannel {
    fn send(&mut self, message: String);
    fn receive(&mut self) -> String;
}

impl<Stream> MessageChannel for WebSocket<Stream>
where
    Stream: std::io::Read + std::io::Write,
{
    fn send(&mut self, message: String) {
        self.write_message(Message::Text(message)).unwrap();
    }

    fn receive(&mut self) -> String {
        let message = self.read_message().unwrap();

        message.into_text().unwrap()
    }
}

pub struct WebsocketServer {}

impl Server for WebsocketServer {
    fn run<T>(self, session_handler: T)
    where
        T: SessionHandler + Clone + Send + 'static,
    {
        let server = TcpListener::bind("127.0.0.1:5007").unwrap();

        for stream in server.incoming() {
            println!("new incoming stream.");

            let session_handler = session_handler.clone();

            spawn(move || {
                let websocket = accept(stream.unwrap()).unwrap();

                let session_handler = session_handler;
                session_handler.handle_session(websocket);
            });
        }
    }
}
