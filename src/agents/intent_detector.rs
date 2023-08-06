use serde::{Deserialize, Serialize};

use crate::{
    conversation::Conversation,
    model_client::{GuidanceRequestBuilder, ModelClient},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
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

    pub async fn detect_intent(
        &self,
        model_client: &(dyn ModelClient + Send + Sync),
        conversation: &Conversation,
    ) -> String {
        let history = conversation.build_history();

        let prompt = self.prompt.replace("{{history}}", &history);

        let intent_objects: Vec<Intent> = self.valid_intents.clone();

        let intent_names: Vec<&str> = self.valid_intents.iter().map(Intent::name).collect();

        let request = GuidanceRequestBuilder::new(prompt)
            .with_object_parameter("intents", &intent_objects)
            .with_parameter_list("intent_names", &intent_names)
            .build();

        let guidance_result = model_client.request_guidance(&request).await;
        let selected_intent = guidance_result.expect_variable("intent");
        selected_intent.into()
    }
}
