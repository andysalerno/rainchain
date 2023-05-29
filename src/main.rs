#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use std::{fs, io::Write};

use env_logger::Env;
use guidance_client::GuidanceClient;
use log::{debug, info};

use crate::{
    model_client::GuidanceRequestBuilder,
    server::{Server, WebsocketServer},
    tools::{web_search::WebSearch, Tool},
};

mod agent;
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

    // let guidance_client = GuidanceClient::new("http://archdesktop.local:8000");
    let guidance_client = GuidanceClient::new("https://notebooksc.jarvislabs.ai/VFf_4YoJ8gJEGdpQJly08ncRAEVFJx3ndc5HcZZ9YocGcmyPON0Y1MdLSduZx4dpIS/proxy/8000");
}

// fn old_run() {
//     // Listens for connections from browsers
//     let server = make_server();

//     let session_handler = Session::new(
//         || Box::new(make_client()),
//         || Box::new(ActionThought::new()),
//     );

//     debug!("Starting server.");
//     server.run(session_handler);
// }

fn make_server() -> impl Server {
    WebsocketServer {}
}

// fn make_client() -> impl ModelClient + Sync + Send {
//     todo!()
//     // WebsocketClient::connect("ws://archdesktop.local:5005/api/v1/stream")
//     // WebsocketClient::connect(
//     //     "wss://resulted-dimension-words-sapphire.trycloudflare.com/api/v1/stream",
//     // )
// }

fn load_prompt_text(prompt_name: &str) -> String {
    let path = format!("src/prompts/{prompt_name}");
    debug!("Reading prompt file: {path}");
    fs::read_to_string(path).expect("Failed to read prompt file")
}
