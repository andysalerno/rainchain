#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions, clippy::too_many_lines)]

use std::{env, fs};

use env_logger::Env;
use guidance_client::GuidanceClient;
use log::debug;
use model_client::ModelClient;

use crate::{
    agents::ThoughtActionAgent,
    server::{Server, WebsocketServer},
    session::AgentSessionHandler,
};

mod agents;
mod conversation;
mod guidance_client;
mod model_client;
mod server;
mod session;
mod tools;

#[tokio::main]
async fn main() {
    // Logging startup
    {
        let env = Env::default().filter_or("RUST_LOG", "rainchain=debug");
        env_logger::init_from_env(env);
        debug!("Starting up.");
    }

    let url = env::args()
        .nth(1)
        .expect("Expected a single argument for the target guidance server url");

    // Listens for connections from browsers
    let server = make_server();

    // let session_handler = Session::new(move || Box::new(make_client(url)));
    let session_handler =
        AgentSessionHandler::new(|| Box::new(ThoughtActionAgent::new(Box::new(make_client(url)))));

    debug!("Starting server.");
    server.run(session_handler).await;
}

fn make_server() -> impl Server {
    WebsocketServer {}
}

fn make_client(url: impl Into<String>) -> impl ModelClient + Sync + Send {
    let url = url.into();
    debug!("Creating client for url: {url}");
    GuidanceClient::new(url)
}

pub(crate) fn load_prompt_text(prompt_name: &str) -> String {
    let path = format!("src/prompts/{prompt_name}");
    debug!("Reading prompt file: {path}");
    fs::read_to_string(path).expect("Failed to read prompt file")
}
