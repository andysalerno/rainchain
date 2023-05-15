use log::{debug, trace};
use serde::Serialize;

use crate::{
    agent::NextStep,
    conversation::Conversation,
    model_client::{ClientRequest, ModelClient},
    server::MessageChannel,
    tools::{noop::Noop, web_search::WebSearch, Tool},
};

use super::Agent;

pub struct ActionThought {
    tools: Vec<Box<dyn Tool>>,
}

#[derive(Serialize)]
struct ToolInfoMessage {
    event: String,
    text: String,
}

impl ActionThought {
    pub fn new() -> Self {
        Self {
            tools: vec![Box::new(WebSearch), Box::new(Noop)],
        }
    }

    fn select_tool(&self, name: &str) -> &dyn Tool {
        self.tools
            .iter()
            .find(|t| t.name() == name)
            .expect("didn't find tool with matching name")
            .as_ref()
    }
}

impl Agent for ActionThought {
    fn handle_assistant_message(
        &self,
        conversation: &mut Conversation,
        channel: &mut dyn MessageChannel,
        model_client: &mut dyn ModelClient,
    ) -> NextStep {
        debug!("ActionThought agent saw message from assistant.");

        if conversation.last_assistant_message().ends_with("</action") {
            debug!("Saw action in last message");

            // Parse message for tool
            let (_thought, action) = extract_thought_action(conversation.last_assistant_message());
            debug!("Extracted action: {action}");
            let (tool_name, input) = extract_tool_and_input(&action);
            debug!("Extracted tool: {tool_name}");
            debug!("Extracted input: {input}");
            let tool = self.select_tool(&tool_name);

            // Invoke tool
            debug!("Invoking tool...");
            let last_user_message = conversation.last_user_message();
            let tool_output = tool.get_output(&input, last_user_message, model_client);
            debug!("Got tool output: {tool_output}");

            // Update with tool output
            let message = conversation.last_assistant_message_mut();
            message.set_baggage("tool_output", &tool_output);
            message.set_baggage("tool_name", &tool_name);
            message.append(&format!(
                ">\n<output>\n{tool_output}\n</output>\n<response>"
            ));

            // Request model to complete response
            model_client.send(ClientRequest::start_predicting(
                conversation.build_full_history(),
            ));

            NextStep::KeepPredicting
        } else if conversation
            .last_assistant_message()
            .ends_with("</response")
        {
            conversation.append_to_last_assistant_message(">");
            conversation.push_eos_token();

            let tool_info = {
                let last_msg = conversation.last_assistant_message_mut();
                last_msg.baggage().get("tool_output")
            };

            if let Some(tool_info) = tool_info {
                // Push the tool output to the UI so the user can inspect it if they choose.
                let tool_info_msg = ToolInfoMessage {
                    event: "ToolInfo".to_owned(),
                    text: tool_info.to_string(),
                };

                debug!("Sending tool info to UI...");
                channel.send(serde_json::to_string(&tool_info_msg).unwrap());
            }

            NextStep::StopPredicting
        } else {
            conversation.push_eos_token();

            NextStep::StopPredicting
        }
    }
}

fn extract_tool_and_input(text: &str) -> (String, String) {
    trace!("Extracting tool and input from: {text}");

    let text = text.trim();

    let tool = text.split('(').next().unwrap().trim();

    if tool == "NONE" {
        (tool.into(), String::new())
    } else {
        let input = text.split('(').nth(1).unwrap().trim().trim_end_matches(')');

        (tool.into(), input.into())
    }
}

/// A hacky and dirty pseudo-xml extractor
fn extract_thought_action(text: &str) -> (String, String) {
    trace!("Extracting action-thought from: {text}");

    let text = text.trim();

    let thought = text
        .split("</thought>")
        .next()
        .and_then(|t| t.split("<thought>").find(|t| !t.is_empty()))
        .map_or("", str::trim);

    let action = text
        .split("<action>")
        .skip(1)
        .find(|t| !t.is_empty())
        .and_then(|t| t.split("</action").find(|t| !t.is_empty()))
        .map_or("", str::trim);

    (thought.into(), action.into())
}

#[cfg(test)]
mod tests {
    use super::extract_tool_and_input;

    #[test]
    fn test_extract_tool_input() {
        let input = "WEB_SEARCH(some input to the search)";

        let (tool, input) = extract_tool_and_input(input);

        assert_eq!(tool, "WEB_SEARCH");
        assert_eq!(input, "some input to the search");
    }

    #[test]
    fn test_extract_tool_input_spaces() {
        let input = " WEB_SEARCH ( some input to the search) ";

        let (tool, input) = extract_tool_and_input(input);

        assert_eq!(tool, "WEB_SEARCH");
        assert_eq!(input, "some input to the search");
    }
}
