use std::collections::HashMap;

use async_trait::async_trait;
use reqwest_eventsource::EventSource;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[async_trait]
pub trait ModelClient {
    async fn request_embeddings(&self, request: &EmbeddingsRequest) -> EmbeddingsResponse;
    async fn request_guidance(&self, request: &GuidanceRequest) -> GuidanceResponse;
    fn request_guidance_stream(&self, request: &GuidanceRequest) -> EventSource;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GuidanceEmbeddingsResponse {
    pub object: String,
    pub data: Vec<Embedding>,
    pub model: String,
}

pub struct GuidanceRequestBuilder {
    template: String,
    parameters: HashMap<String, serde_json::Value>,
}

impl GuidanceRequestBuilder {
    pub fn new(template: impl Into<String>) -> Self {
        Self {
            template: template.into(),
            parameters: HashMap::new(),
        }
    }

    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let key = key.into();
        let value = value.into();

        let value = json!(value);

        self.parameters.insert(key, value);

        self
    }

    pub fn with_parameter_list(mut self, key: impl Into<String>, value: &[&str]) -> Self {
        let key = key.into();
        let value = json!(value);

        self.parameters.insert(key, value);

        self
    }

    pub fn build(self) -> GuidanceRequest {
        GuidanceRequest {
            template: self.template,
            parameters: self.parameters,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuidanceRequest {
    template: String,
    parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GuidanceResponse {
    pub text: String,
    pub variables: HashMap<String, String>,
}

impl GuidanceResponse {
    pub fn expect_variable(&self, key: &str) -> &str {
        self.variables
            .get(key)
            .expect("Expected to find the key, but did not.")
    }

    pub fn variable(&self, key: &str) -> Option<&str> {
        self.variables.get(key).map(String::as_str)
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn apply_delta(&mut self, delta: GuidanceResponse) {
        self.text.push_str(delta.text());

        for (k, v) in delta.variables {
            if let Some(current) = self.variables.get_mut(&k) {
                current.push_str(&v);
            } else {
                self.variables.insert(k, v);
            }
        }
    }

    pub fn new() -> Self {
        GuidanceResponse {
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GuidanceEmbeddingsRequest {
    input: Vec<String>,
}

#[derive(Default)]
pub struct GuidanceEmbeddingsRequestBuilder {
    input: Vec<String>,
}

impl GuidanceEmbeddingsRequestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_input(self, input: impl Into<String>) -> Self {
        self.add_inputs([input])
    }

    pub fn add_inputs<TIter, TStr>(mut self, inputs: TIter) -> Self
    where
        TIter: IntoIterator<Item = TStr>,
        TStr: Into<String>,
    {
        let owned: Vec<String> = inputs.into_iter().map(std::convert::Into::into).collect();
        self.input.extend(owned);

        self
    }

    pub fn build(self) -> GuidanceEmbeddingsRequest {
        GuidanceEmbeddingsRequest { input: self.input }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingsRequest {
    pub input: Vec<String>,
}

impl EmbeddingsRequest {
    pub fn new(input: Vec<String>) -> Self {
        Self { input }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingsResponse {
    pub object: String,
    pub data: Vec<Embedding>,
    pub model: String,
}

impl EmbeddingsResponse {
    pub fn take_embeddings(self) -> Vec<Embedding> {
        self.data
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    object: String,
    embedding: Vec<f32>,
    index: usize,
}

impl Embedding {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn embedding(&self) -> &[f32] {
        self.embedding.as_ref()
    }
}
