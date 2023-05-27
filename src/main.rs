#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use std::{fs, io::Write};

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

    // let guidance_client = GuidanceClient::new("http://archdesktop.local:8000");
    let guidance_client = GuidanceClient::new("https://notebooksc.jarvislabs.ai/VFf_4YoJ8gJEGdpQJly08ncRAEVFJx3ndc5HcZZ9YocGcmyPON0Y1MdLSduZx4dpIS/proxy/8000");

    let prompt_preamble = load_prompt_text("guider_preamble.txt");
    let chat_start_sep = "\nBegin!\n"; // hack
    let prompt_chat = load_prompt_text("guider_chat.txt");
    let generate_question_prompt = load_prompt_text("generate_question.txt");

    let mut history = String::new();

    loop {
        // Get user's input:
        let mut user_input = String::new();
        {
            print!("Type your message: ");
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut user_input).unwrap();
            user_input = user_input.trim().to_string();
        }

        // Hack: we need to manually replace {{history}} first, because that value
        // is itself templated, and guidance only performs template replacement once
        let prompt_chat: String = prompt_chat.replace("{{preamble}}", &prompt_preamble);

        let request = GuidanceRequestBuilder::new(&prompt_chat)
            // .with_parameter("preamble", &prompt_preamble)
            .with_parameter("history", history)
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
        let output = if action == "NONE" {
            String::new()
        } else {
            tool.get_output(action_input, action_input, &guidance_client)
        };

        info!("Got tool output:\n{output}");

        // Send OUTPUT back to model and let it continue:
        let ongoing_chat = response.text();
        let request = GuidanceRequestBuilder::new(ongoing_chat)
            .with_parameter("output", output)
            .build();
        let response = guidance_client.get_response(&request);
        info!("Got response: {response:?}");

        history = response.text().to_owned();
        // history = history.drain(preamble_len..).collect();
        history = history
            .split(chat_start_sep)
            .nth(1)
            .expect("could not find chat history")
            .to_owned();
        history = history.trim().to_owned();
        // Clear out any "output" sections from history, to save up space in our LLM context
        // major hack:
        history = history
            .lines()
            .filter(|l| !l.trim_start().starts_with("[WEB_RESULT"))
            .collect::<Vec<&str>>()
            .join("\n");
        history.push('\n');
        info!("New history:\n{history}");
    }
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
