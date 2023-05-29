use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{debug, info};
use reqwest::{Client, Url};
use reqwest_eventsource::{Event, EventSource, RequestBuilderExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, time::Duration};

use crate::model_client::{
    Embedding, EmbeddingsResponse, GuidanceEmbeddingsRequest, GuidanceEmbeddingsRequestBuilder,
    GuidanceEmbeddingsResponse, GuidanceRequest, GuidanceResponse, ModelClient,
};

pub struct GuidanceClient {
    uri: String,
}

impl GuidanceClient {
    pub fn new(uri: impl Into<String>) -> Self {
        Self { uri: uri.into() }
    }

    pub async fn get_response(&self, request: &GuidanceRequest) -> GuidanceResponse {
        let client = reqwest::Client::new();

        let url = Url::parse(&format!("{}/chat", self.uri)).expect("Failed to parse guidance url");

        let body =
            serde_json::to_string(request).expect("Failed to parse guidance request to json");

        let mut stream = client.post(url).body(body).eventsource().unwrap();

        let mut final_response = GuidanceResponse::new();

        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => match event {
                    Event::Open => info!("got open event"),
                    Event::Message(m) => {
                        let response: GuidanceResponse = serde_json::from_str(&m.data)
                            .expect("response was not in the expected format");
                        info!("got message: {response:?}");

                        final_response.text.push_str(response.text());

                        for (k, v) in response.variables {
                            if let Some(current) = final_response.variables.get_mut(&k) {
                                current.push_str(&v);
                            } else {
                                final_response.variables.insert(k, v);
                            }
                        }
                    }
                },
                _ => break,
            }
        }

        info!("done. final:\n{:#?}", final_response);

        return final_response;
    }

    pub async fn get_embeddings(
        &self,
        request: &GuidanceEmbeddingsRequest,
    ) -> GuidanceEmbeddingsResponse {
        let client = reqwest::Client::new();

        let url = Url::parse(&format!("{}/embeddings", self.uri))
            .expect("Failed to parse guidance embeddings url");

        let body = serde_json::to_string(request)
            .expect("Failed to parse guidance embeddings request to json");

        info!("Sending guidance embeddings request to {url}...");
        let json = client
            .post(url)
            .body(body)
            .timeout(Duration::from_secs(120))
            .send()
            .await
            .expect("Failed to send guidance embeddings request")
            .text()
            .await
            .expect("Expected text response");
        info!("...Got response.");

        let parsed: GuidanceEmbeddingsResponse = serde_json::from_str(&json).unwrap();

        parsed
    }
}

#[async_trait]
impl ModelClient for GuidanceClient {
    async fn request_embeddings(
        &self,
        request: &crate::model_client::EmbeddingsRequest,
    ) -> crate::model_client::EmbeddingsResponse {
        let mut mapped_request = GuidanceEmbeddingsRequestBuilder::default();

        for r in &request.input {
            mapped_request = mapped_request.add_input(r);
        }

        let response = self.get_embeddings(&mapped_request.build()).await;

        EmbeddingsResponse {
            object: response.object,
            data: response.data,
            model: response.model,
        }
    }
}
