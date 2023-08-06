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

    pub fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
    }

    pub fn build_history(&self) -> String {
        let mut result = String::new();

        for message in self.messages() {
            let (role_start, role_end) = match message {
                ChatMessage::User(_) => ("{{~#user~}}", "{{~/user}}"),
                ChatMessage::Assistant(_) => ("{{~#assistant}}", "{{~/assistant}}"),
                ChatMessage::System(_) => ("<<SYS>>", "<</SYS>>"),
            };

            let text = message.text();

            result.push_str(&format!("{role_start}{text}{role_end}"));
        }

        result
    }
}
