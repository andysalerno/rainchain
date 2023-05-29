#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions, clippy::too_many_lines)]

use env_logger::Env;
use guidance_client::GuidanceClient;
use log::debug;
use model_client::ModelClient;

use crate::{
    server::{Server, WebsocketServer},
    session::Session,
};

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

    // Listens for connections from browsers
    let server = make_server();

    let session_handler = Session::new(|| Box::new(make_client()));

    debug!("Starting server.");
    server.run(session_handler).await;
}

fn make_server() -> impl Server {
    WebsocketServer {}
}

fn make_client() -> impl ModelClient + Sync + Send {
    GuidanceClient::new("https://notebooksc.jarvislabs.ai/VFf_4YoJ8gJEGdpQJly08ncRAEVFJx3ndc5HcZZ9YocGcmyPON0Y1MdLSduZx4dpIS/proxy/8000")
}
