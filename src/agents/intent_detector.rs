use crate::model_client::{GuidanceRequestBuilder, ModelClient};

pub(crate) struct Intent {
    name: String,
    description: String,
}

impl Intent {
    pub(crate) fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }

    pub(crate) fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub(crate) fn description(&self) -> &str {
        self.description.as_ref()
    }
}

pub(crate) struct IntentDetector {
    valid_intents: Vec<Intent>,
    prompt: String,
}

impl IntentDetector {
    pub(crate) fn new(valid_intents: Vec<Intent>, prompt: String) -> Self {
        Self {
            valid_intents,
            prompt,
        }
    }

    pub async fn detect_intent(&self, model_client: &(dyn ModelClient + Send + Sync)) -> String {
        let request = GuidanceRequestBuilder::new(&self.prompt)
            .with_parameter("user_input", "blah")
            .with_parameter_list("valid_actions", &["WEB_SEARCH", "NONE"])
            .build();

        model_client.request_guidance(&request).await;
        "intent".into()
    }
}
