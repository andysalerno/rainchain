use log::{debug, trace};

use crate::{
    conversation::Conversation,
    server::MessageChannel,
    tools::{web_search::WebSearch, Tool},
};

use super::Agent;

pub struct ActionThought {
    tools: Vec<Box<dyn Tool>>,
}

impl ActionThought {
    pub fn new() -> Self {
        Self {
            tools: vec![Box::new(WebSearch)],
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
    ) {
        debug!("ActionThought agent saw message from assistant.");

        if conversation.last_assistant_message().ends_with("</action") {
            debug!("Saw action in last message");
            let (thought, action) = extract_thought_action(conversation.last_assistant_message());

            debug!("Extracted action: {action}");

            let (tool_name, input) = extract_tool_and_input(&action);

            debug!("Extracted tool: {tool_name}");
            debug!("Extracted input: {input}");

            let tool = self.select_tool(&tool_name);

            debug!("Invoking tool...");
            let tool_output = tool.get_output(&input);
        } else {
            conversation.push_eos_token();
        }
    }
}

fn extract_tool_and_input(text: &str) -> (String, String) {
    trace!("Extracting tool and input from: {text}");

    let text = text.trim();

    let tool = text.split('(').next().unwrap().trim();

    let input = text.split('(').nth(1).unwrap().trim().trim_end_matches(')');

    (tool.into(), input.into())
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
