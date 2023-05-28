use serde::{Deserialize, Serialize};

pub trait ModelClient {
    fn request_embeddings(&self, request: &EmbeddingsRequest) -> EmbeddingsResponse;
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
