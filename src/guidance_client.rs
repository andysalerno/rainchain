use log::{debug, info};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

use crate::model_client::{Embedding, EmbeddingsResponse, ModelClient};

#[derive(Debug, Serialize, Deserialize)]
pub struct GuidanceRequest {
    template: String,
    parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuidanceResponse {
    text: String,
    variables: HashMap<String, String>,
}

impl GuidanceResponse {
    pub fn expect_variable(&self, key: &str) -> &str {
        self.variables
            .get(key)
            .expect("Expected to find the key, but did not.")
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

pub struct GuidanceClient {
    uri: String,
}

impl GuidanceClient {
    pub fn new(uri: impl Into<String>) -> Self {
        Self { uri: uri.into() }
    }

    pub fn get_response(&self, request: &GuidanceRequest) -> GuidanceResponse {
        let client = reqwest::blocking::Client::new();

        let url = Url::parse(&format!("{}/chat", self.uri)).expect("Failed to parse guidance url");

        let body =
            serde_json::to_string(request).expect("Failed to parse guidance request to json");

        info!("Sending guidance request to {url}...");
        debug!("...Body: {body}");
        let json = client
            .post(url)
            .body(body)
            .send()
            .expect("Failed to send guidance request")
            .text()
            .expect("Expected text response");
        info!("...Got response.");
        debug!("Response: {json}");

        let parsed: GuidanceResponse = serde_json::from_str(&json).unwrap();

        parsed
    }

    pub fn get_embeddings(
        &self,
        request: &GuidanceEmbeddingsRequest,
    ) -> GuidanceEmbeddingsResponse {
        let client = reqwest::blocking::Client::new();

        let url = Url::parse(&format!("{}/embeddings", self.uri))
            .expect("Failed to parse guidance embeddings url");

        let body = serde_json::to_string(request)
            .expect("Failed to parse guidance embeddings request to json");

        info!("Sending guidance embeddings request to {url}...");
        debug!("...Body: {body}");
        let json = client
            .post(url)
            .body(body)
            .send()
            .expect("Failed to send guidance embeddings request")
            .text()
            .expect("Expected text response");
        info!("...Got response.");

        let parsed: GuidanceEmbeddingsResponse = serde_json::from_str(&json).unwrap();

        parsed
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

#[derive(Serialize, Deserialize, Debug)]
pub struct GuidanceEmbeddingsResponse {
    object: String,
    data: Vec<Embedding>,
    model: String,
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

impl ModelClient for GuidanceClient {
    fn receive(&mut self) -> crate::model_client::ServerResponse {
        todo!("hack - this will never be implemented")
    }

    fn send(&mut self, message: crate::model_client::ClientRequest) {
        todo!("hack - this will never be implemented")
    }

    fn request_embeddings(
        &self,
        request: &crate::model_client::EmbeddingsRequest,
    ) -> crate::model_client::EmbeddingsResponse {
        let mut mapped_request = GuidanceEmbeddingsRequestBuilder::default();

        for r in &request.input {
            mapped_request = mapped_request.add_input(r);
        }

        let response = self.get_embeddings(&mapped_request.build());

        EmbeddingsResponse {
            object: response.object,
            data: response.data,
            model: response.model,
        }
    }
}
