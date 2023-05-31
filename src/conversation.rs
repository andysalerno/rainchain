#[derive(Default, Clone, Debug)]
pub struct Conversation {
    full_history: String,
    user_messages: Vec<String>,
}

impl Conversation {
    pub fn new() -> Self {
        Self {
            full_history: String::new(),
            user_messages: Vec::new(),
        }
    }

    pub fn full_history(&self) -> &str {
        &self.full_history
    }

    pub fn update(&mut self, history: impl Into<String>) {
        let mut history = history.into();

        let first_user_input = self
            .user_messages
            .first()
            .expect("Expected at least one input from the user before this point.");

        let first_user_input_pos = history
            .find(first_user_input)
            .expect("expected to see the user's first input in the full history");

        // This hack needs to be replaced:
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

        self.full_history = history;
    }

    pub fn add_user_message(&mut self, message: impl Into<String>) {
        let message = message.into();
        self.user_messages.push(message);
    }
}
