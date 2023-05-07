use std::sync::Arc;

use client::{Client, WebsocketClient};

use crate::{
    server::{Server, WebsocketServer},
    session::Session,
};

mod client;
mod conversation;
mod server;
mod session;

fn main() {
    println!("Hello, world!");

    // Listens for connections from browsers
    let server = make_server();

    let session_handler = Session::new(|| Box::new(make_client()));

    server.run(session_handler);
}

fn make_server() -> impl Server {
    WebsocketServer {}
}

fn make_client() -> impl Client<String> + Sync + Send {
    WebsocketClient {}
}
