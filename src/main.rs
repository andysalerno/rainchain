#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use std::fs;

use env_logger::Env;
use guidance_client::{GuidanceClient, GuidanceRequestBuilder};
use log::{debug, info};
use model_client::{ModelClient, WebsocketClient};

use crate::{
    agent::action_thought::ActionThought,
    server::{Server, WebsocketServer},
    session::Session,
    tools::{web_search::WebSearch, Tool},
};

mod agent;
mod conversation;
mod guidance_client;
mod model_client;
mod server;
mod session;
mod tools;

fn main() {
    // Logging startup
    {
        let env = Env::default().filter_or("RUST_LOG", "rainchain=debug");
        env_logger::init_from_env(env);
        debug!("Starting up.");
    }

    let guidance_client = GuidanceClient::new("http://archdesktop.local:8000");

    let prompt_preamble = load_prompt_text("guider_preamble.txt");
    let prompt_chat = load_prompt_text("guider_chat.txt");

    let user_input = "What's the best smartphone I can buy today?";

    let request = GuidanceRequestBuilder::new(prompt_chat)
        .with_parameter("preamble", &prompt_preamble)
        .with_parameter("history", "")
        .with_parameter("user_input", user_input)
        .with_parameter_list("valid_actions", &["WEB_SEARCH", "NONE"])
        .build();

    // The first response will have THOUGHT and ACTION filled out.
    let response = guidance_client.get_response(&request);
    let action = response.expect_variable("action").trim();
    let action_input = response.expect_variable("action_input").trim();
    info!("Got response: {response:?}");

    // Now we must provide OUTPUT:
    let tool = WebSearch;
    let output = tool.get_output(action_input, action_input, &guidance_client);
    info!("Got tool output:\n{output}");

    // Send OUTPUT back to model and let it continue:
    let ongoing_chat = response.text();
    let request = GuidanceRequestBuilder::new(ongoing_chat)
        .with_parameter("output", output)
        .build();
    let response = guidance_client.get_response(&request);
    info!("Got response: {response:?}");
}

fn old_run() {
    // // Listens for connections from browsers
    // let server = make_server();

    // let session_handler = Session::new(
    //     || Box::new(make_client()),
    //     || Box::new(ActionThought::new()),
    // );

    // debug!("Starting server.");
    // server.run(session_handler);
}

fn make_server() -> impl Server {
    WebsocketServer {}
}

fn make_client() -> impl ModelClient + Sync + Send {
    WebsocketClient::connect("ws://archdesktop.local:5005/api/v1/stream")
    // WebsocketClient::connect(
    //     "wss://resulted-dimension-words-sapphire.trycloudflare.com/api/v1/stream",
    // )
}

fn load_prompt_text(prompt_name: &str) -> String {
    let path = format!("src/prompts/{prompt_name}");
    debug!("Reading prompt file: {path}");
    fs::read_to_string(path).expect("Failed to read prompt file")
}
