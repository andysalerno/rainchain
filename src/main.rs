#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use env_logger::Env;
use log::debug;
use model_client::{Client, WebsocketClient};

use crate::{
    agent::action_thought::ActionThought,
    server::{Server, WebsocketServer},
    session::Session,
};

mod agent;
mod conversation;
mod model_client;
mod server;
mod session;
mod tools;

fn main() {
    // Logging startup
    {
        let env = Env::default().filter_or("RUST_LOG", "debug");
        env_logger::init_from_env(env);
        debug!("Starting up.");
    }

    // Listens for connections from browsers
    let server = make_server();

    let session_handler = Session::new(
        || Box::new(make_client()),
        || Box::new(ActionThought::new()),
    );

    debug!("Starting server.");
    server.run(session_handler);
}

fn make_server() -> impl Server {
    WebsocketServer {}
}

fn make_client() -> impl Client + Sync + Send {
    WebsocketClient::connect("ws://archdesktop.local:5005/api/v1/stream")
}
