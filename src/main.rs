#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use std::{fs, io::Write};

use env_logger::Env;
use guidance_client::{GuidanceClient, GuidanceRequestBuilder};
use log::{debug, info};
use model_client::ModelClient;

use crate::{
    server::{Server, WebsocketServer},
    tools::{web_search::WebSearch, Tool},
};

mod agent;
mod conversation;
mod guidance_client;
mod model_client;
mod server;
// mod session;
mod tools;

#[tokio::main]
async fn main() {
    // Logging startup
    {
        let env = Env::default().filter_or("RUST_LOG", "rainchain=debug");
        env_logger::init_from_env(env);
        debug!("Starting up.");
    }

    // old_run();

    // let guidance_client = GuidanceClient::new("http://archdesktop.local:8000");
    let guidance_client = GuidanceClient::new("https://notebooksc.jarvislabs.ai/VFf_4YoJ8gJEGdpQJly08ncRAEVFJx3ndc5HcZZ9YocGcmyPON0Y1MdLSduZx4dpIS/proxy/8000");

    let prompt_preamble = load_prompt_text("guider_preamble.txt");
    let prompt_chat = load_prompt_text("guider_chat.txt");

    let mut history = String::new();

    let mut first_user_input: Option<String> = None;

    loop {
        // Get user's input:
        let mut user_input = String::new();
        {
            print!("Type your message: ");
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut user_input).unwrap();
            user_input = user_input.trim().to_string();
        }

        if first_user_input.is_none() {
            first_user_input = Some(user_input.clone());
        }

        let first_user_input = first_user_input.as_ref().unwrap();

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
        let response = guidance_client.get_response(&request).await;
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
        let response = guidance_client.get_response(&request).await;
        info!("Got response: {response:?}");

        // Update history
        {
            history = response.text().to_owned();

            let first_user_input_pos = history
                .find(first_user_input)
                .expect("expected to see the user's first input in the history");

            let preceding_newline_pos = history[0..first_user_input_pos]
                .rfind('\n')
                .expect("expected to find a newline before the user's message");

            history = history.split_off(preceding_newline_pos);

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
