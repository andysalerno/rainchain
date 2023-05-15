use std::collections::HashMap;

pub struct Message {
    message_num: usize,
    role: String,
    text: String,
    baggage: HashMap<String, String>,
}

impl Message {
    fn new(message_num: usize, role: String, text: String) -> Self {
        Self {
            message_num,
            role,
            text,
            baggage: HashMap::new(),
        }
    }

    pub fn append(&mut self, text: &str) {
        self.text.push_str(text);
    }

    pub fn baggage(&self) -> &HashMap<String, String> {
        &self.baggage
    }

    pub fn set_baggage(&mut self, key: &str, value: &str) {
        self.baggage.insert(key.into(), value.into());
    }

    fn to_string(&self) -> String {
        let trimmed = self.text.trim();
        let role = &self.role;
        format!("{role}: {trimmed}").trim().to_owned()
    }
}

pub struct Conversation {
    base_prompt: String,
    user_messages: Vec<Message>,
    assistant_messages: Vec<Message>,
    user_role: String,
    bot_role: String,
    eos_token: String,
}

impl Conversation {
    pub fn new(base_prompt: impl Into<String>) -> Self {
        Self {
            base_prompt: base_prompt.into(),
            user_messages: Vec::new(),
            assistant_messages: Vec::new(),
            user_role: "USER".into(),
            bot_role: "ASSISTANT".into(),
            eos_token: "</s>".into(),
        }
    }

    pub fn add_user_message(&mut self, message: String) {
        let num = self.message_count() + 1;
        self.user_messages
            .push(Message::new(num, self.user_role.clone(), message));
    }

    pub fn add_assistant_message(&mut self, message: String) {
        let num = self.message_count() + 1;
        self.assistant_messages
            .push(Message::new(num, self.bot_role.clone(), message));
    }

    pub fn push_eos_token(&mut self) {
        let eos_token = self.eos_token.clone();
        self.last_assistant_message_mut().append(&eos_token);
    }

    pub fn append_to_last_assistant_message(&mut self, text: &str) {
        self.last_assistant_message_mut().append(text);
    }

    pub fn last_assistant_message(&self) -> &str {
        let last_message = self.assistant_messages.last().unwrap();
        &last_message.text
    }

    pub fn last_assistant_message_mut(&mut self) -> &mut Message {
        self.assistant_messages
            .last_mut()
            .expect("Expected at least one message from the assistant")
    }

    pub fn last_user_message(&self) -> &str {
        let last_message = self.user_messages.last().unwrap();
        &last_message.text
    }

    pub fn build_full_history(&self) -> String {
        let mut combined = String::new();

        combined.push_str(&self.base_prompt);

        let mut all_messages = self
            .user_messages
            .iter()
            .chain(self.assistant_messages.iter())
            .collect::<Vec<_>>();
        all_messages.sort_unstable_by_key(|m| m.message_num);

        for message in all_messages {
            combined.push('\n');
            combined.push_str(&message.to_string());
        }

        combined
    }

    fn message_count(&self) -> usize {
        self.user_messages.len() + self.assistant_messages.len()
    }
}
