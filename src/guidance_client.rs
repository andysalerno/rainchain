use async_trait::async_trait;
use futures::stream::StreamExt;

use futures_util::Stream;
use log::info;
use reqwest::Url;
use reqwest_eventsource::{Event, EventSource, RequestBuilderExt};

use std::{collections::HashMap, future, pin::Pin, time::Duration};

use crate::model_client::{
    EmbeddingsResponse, GuidanceEmbeddingsRequest, GuidanceEmbeddingsRequestBuilder,
    GuidanceRequest, GuidanceResponse, MemoryGetRequest, MemoryGetResponse, MemoryStoreRequest,
    ModelClient,
};

pub struct GuidanceClient {
    uri: String,
}

impl GuidanceClient {
    pub fn new(uri: impl Into<String>) -> Self {
        Self { uri: uri.into() }
    }

    pub fn get_response_stream(&self, request: &GuidanceRequest) -> EventSource {
        let client = reqwest::Client::new();

        let url = Url::parse(&format!("{}/chat", self.uri)).expect("Failed to parse guidance url");

        let body =
            serde_json::to_string(request).expect("Failed to parse guidance request to json");

        client.post(url).body(body).eventsource().unwrap()
    }

    pub async fn get_response(&self, request: &GuidanceRequest) -> GuidanceResponse {
        let mut stream = self.get_response_stream(request);

        let mut final_response = GuidanceResponse::new();

        while let Some(event) = stream.next().await {
            info!("got some event");
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

        final_response
    }

    async fn store_memory(&self, request: &MemoryStoreRequest) {
        let client = reqwest::Client::new();

        let url = Url::parse(&format!("{}/memory", self.uri))
            .expect("Failed to parse guidance memory url");

        let body = serde_json::to_string(request)
            .expect("Failed to parse guidance memory request to json");

        info!("Sending guidance memory request to {url}...");
        info!("{body}");

        client
            .post(url)
            .body(body)
            .timeout(Duration::from_secs(120))
            .send()
            .await
            .expect("Failed to send guidance memory request")
            .text()
            .await
            .expect("Expected text response");
        info!("...Got response.");
    }

    async fn get_memory_response(&self, request: &MemoryGetRequest) -> MemoryGetResponse {
        let client = reqwest::Client::new();

        let params = {
            let mut map = HashMap::<String, String>::new();

            map.insert("query".to_owned(), request.query.clone());

            map
        };

        let url = Url::parse_with_params(&format!("{}/memory", self.uri), &params)
            .expect("Failed to parse guidance embeddings url");

        info!("Sending request to: {url}");

        let response = client.get(url).send().await;
        let text = response
            .expect("Failed to get memory")
            .text()
            .await
            .expect("Expected text in response");

        info!("Get memory response:\n{text}");

        let parsed: MemoryGetResponse = serde_json::from_str(&text).unwrap();

        info!("Memory response: {parsed:?}");

        parsed
    }

    pub async fn get_embeddings(&self, request: &GuidanceEmbeddingsRequest) -> EmbeddingsResponse {
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

        let parsed: EmbeddingsResponse = serde_json::from_str(&json).unwrap();

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

        self.get_embeddings(&mapped_request.build()).await
    }

    async fn request_guidance(&self, request: &GuidanceRequest) -> GuidanceResponse {
        self.get_response(request).await
    }

    async fn request_memory(&self, request: &MemoryGetRequest) -> MemoryGetResponse {
        self.get_memory_response(request).await
    }

    async fn store_memory(&self, request: &MemoryStoreRequest) {
        self.store_memory(request).await;
    }

    fn request_guidance_stream(
        &self,
        request: &GuidanceRequest,
    ) -> Box<dyn Stream<Item = Option<GuidanceResponse>> + Send + Unpin> {
        let event_source = self.get_response_stream(request);

        let mapped = event_source
            // First, filter out "Open", but preserve errors and everything else
            .filter_map(|message| {
                let filter = match message {
                    Ok(Event::Open) => None,
                    e => Some(e),
                };

                future::ready(filter)
            })
            .map(|message| {
                let Ok(message) = message else {
                    // At this point, Errors become None
                    return None;
                };

                let Event::Message(message) = message else {
                    panic!("This is never possible; we filtered out Open.");
                };

                info!("Saw data: {}", message.data);

                let delta: GuidanceResponse = serde_json::from_str(&message.data)
                    .expect("response was not in the expected GuidanceResponse format");

                Some(delta)
            });

        Box::new(mapped)
    }
}
