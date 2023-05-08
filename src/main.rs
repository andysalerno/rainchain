#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use model_client::{Client, WebsocketClient};

use crate::{
    server::{Server, WebsocketServer},
    session::Session,
};

mod conversation;
mod model_client;
mod server;
mod session;

fn main() {
    // Listens for connections from browsers
    let server = make_server();

    let session_handler = Session::new(|| Box::new(make_client()));

    server.run(session_handler);
}

fn make_server() -> impl Server {
    WebsocketServer {}
}

fn make_client() -> impl Client + Sync + Send {
    WebsocketClient::connect("ws://archdesktop.local:5005/api/v1/stream")
}
