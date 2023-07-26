#[derive(Default, Clone, Debug)]
pub struct Conversation {
    messages: Vec<ChatMessage>,
}

#[derive(Clone, Debug)]
pub enum ChatMessage {
    User(String),
    Assistant(String),
    System(String),
}

impl ChatMessage {
    pub fn text(&self) -> &str {
        match self {
            ChatMessage::Assistant(s) | ChatMessage::User(s) | ChatMessage::System(s) => s,
        }
    }

    /// Returns `true` if the chat message is [`User`].
    ///
    /// [`User`]: ChatMessage::User
    #[must_use]
    pub fn is_user(&self) -> bool {
        matches!(self, Self::User(..))
    }

    /// Returns `true` if the chat message is [`Assistant`].
    ///
    /// [`Assistant`]: ChatMessage::Assistant
    #[must_use]
    pub fn is_assistant(&self) -> bool {
        matches!(self, Self::Assistant(..))
    }

    /// Returns `true` if the chat message is [`System`].
    ///
    /// [`System`]: ChatMessage::System
    #[must_use]
    pub fn is_system(&self) -> bool {
        matches!(self, Self::System(..))
    }
}

impl Conversation {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    // pub fn update(&mut self, history: impl Into<String>) {
    //     let mut history = history.into();

    //     let first_user_input = self
    //         .messages
    //         .iter()
    //         .find(|m| m.is_user())
    //         .map(ChatMessage::text)
    //         .expect("Expected at least one input from the user before this point.");

    //     // let first_user_input = self
    //     //     .user_messages
    //     //     .first()
    //     //     .expect("Expected at least one input from the user before this point.");

    //     let first_user_input_pos = history
    //         .find(first_user_input)
    //         .expect("expected to see the user's first input in the full history");

    //     // This hack needs to be replaced:
    //     let preceding_newline_pos = history[0..first_user_input_pos]
    //         .rfind('\n')
    //         .expect("expected to find a newline before the user's message");

    //     history = history.split_off(preceding_newline_pos);

    //     history = history.trim().to_owned();

    //     // Clear out any "output" sections from history, to save up space in our LLM context
    //     // major hack:
    //     history = history
    //         .lines()
    //         .filter(|l| !l.trim_start().starts_with("[WEB_RESULT"))
    //         .filter(|l| {
    //             !l.trim_start()
    //                 .starts_with("*I will use the following results")
    //         })
    //         .collect::<Vec<&str>>()
    //         .join("\n");
    //     history.push('\n');

    //     self.full_history = history;
    // }

    pub fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
    }

    // pub fn add_user_message(&mut self, message: impl Into<String>) {
    //     let message = message.into();
    //     self.user_messages.push(message);
    // }
}
