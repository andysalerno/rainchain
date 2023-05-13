use log::{debug, trace};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::net::TcpStream;
use std::thread;

use tungstenite::{stream::MaybeTlsStream, Message, WebSocket};

pub trait ModelClient {
    fn receive(&mut self) -> ServerResponse;
    fn send(&mut self, message: ClientRequest);
    fn request_embeddings(&self, request: &EmbeddingsRequest) -> EmbeddingsResponse;
}

pub struct WebsocketClient {
    connection: WebSocket<MaybeTlsStream<TcpStream>>,
}

impl WebsocketClient {
    pub fn connect(ws_uri: impl AsRef<str>) -> Self {
        let (connection, _) = tungstenite::connect(ws_uri.as_ref()).unwrap();

        WebsocketClient { connection }
    }
}

impl ModelClient for WebsocketClient {
    fn receive(&mut self) -> ServerResponse {
        let json = loop {
            if let Ok(message) = self.connection.read_message() {
                break message.into_text().unwrap();
            }

            thread::yield_now();
        };

        trace!("received from model: {json}");
        serde_json::from_str(&json).unwrap()
    }

    fn send(&mut self, message: ClientRequest) {
        let json = serde_json::to_string(&message).unwrap();
        trace!("Sending json to client: {json}");
        let ws_message = Message::text(json);
        self.connection.write_message(ws_message).unwrap();
    }

    fn request_embeddings(&self, request: &EmbeddingsRequest) -> EmbeddingsResponse {
        let client = reqwest::blocking::Client::new();

        let url = Url::parse("http://archdesktop.local:5001/v1/embeddings")
            .expect("Failed to parse target embeddings url");

        let body = serde_json::to_string(request).expect("Failed to parse request to json");

        let json = client
            .post(url)
            .body(body)
            .send()
            .expect("Failed tos end embeddings request")
            .text()
            .expect("Expected text response");

        let parsed: EmbeddingsResponse = serde_json::from_str(&json).unwrap();

        parsed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClientRequest {
    StartPredicting {
        prompt: String,
        max_new_tokens: usize,
        do_sample: bool,
        temperature: f32,
        top_p: f32,
        typical_p: f32,
        repetition_penalty: f32,
        encoder_repetition_penalty: f32,
        top_k: usize,
        min_length: usize,
        no_repeat_ngram_size: usize,
        num_beams: usize,
        penalty_alpha: f32,
        length_penalty: f32,
        early_stopping: bool,
        seed: i32,
        add_bos_token: bool,
        truncation_length: usize,
        ban_eos_token: bool,
        skip_special_tokens: bool,
        stopping_strings: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingsRequest {
    input: Vec<String>,
}

impl EmbeddingsRequest {
    pub fn new(input: Vec<String>) -> Self {
        Self { input }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingsResponse {
    object: String,
    data: Vec<Embedding>,
    model: String,
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

// generate_params = {
//     'max_new_tokens': int(body.get('max_new_tokens', body.get('max_length', 200))),
//     'do_sample': bool(body.get('do_sample', True)),
//     'temperature': float(body.get('temperature', 0.5)),
//     'top_p': float(body.get('top_p', 1)),
//     'typical_p': float(body.get('typical_p', body.get('typical', 1))),
//     'repetition_penalty': float(body.get('repetition_penalty', body.get('rep_pen', 1.1))),
//     'encoder_repetition_penalty': float(body.get('encoder_repetition_penalty', 1.0)),
//     'top_k': int(body.get('top_k', 0)),
//     'min_length': int(body.get('min_length', 0)),
//     'no_repeat_ngram_size': int(body.get('no_repeat_ngram_size', 0)),
//     'num_beams': int(body.get('num_beams', 1)),
//     'penalty_alpha': float(body.get('penalty_alpha', 0)),
//     'length_penalty': float(body.get('length_penalty', 1)),
//     'early_stopping': bool(body.get('early_stopping', False)),
//     'seed': int(body.get('seed', -1)),
//     'add_bos_token': bool(body.get('add_bos_token', True)),
//     'truncation_length': int(body.get('truncation_length', 2048)),
//     'ban_eos_token': bool(body.get('ban_eos_token', False)),
//     'skip_special_tokens': bool(body.get('skip_special_tokens', True)),
//     'custom_stopping_strings': '',  # leave this blank
//     'stopping_strings': body.get('stopping_strings', []),
// }

impl ClientRequest {
    pub fn start_predicting(prompt: String) -> Self {
        ClientRequest::StartPredicting {
            prompt,
            max_new_tokens: 200,
            do_sample: true,
            temperature: 0.7,
            top_p: 0.5,
            typical_p: 1.,
            repetition_penalty: 1.1,
            encoder_repetition_penalty: 1.1,
            top_k: 0,
            min_length: 0,
            no_repeat_ngram_size: 0,
            num_beams: 1,
            penalty_alpha: 0.,
            length_penalty: 1.,
            early_stopping: false,
            seed: -1,
            add_bos_token: true,
            truncation_length: 2048,
            ban_eos_token: false,
            skip_special_tokens: true,
            stopping_strings: ["</s>".into(), "\n</action>".into()].into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictParameters {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ServerResponse {
    Tokens {
        event: String,
        message_num: usize,
        text: String,
    },
    StreamEnd {
        event: String,
        message_num: usize,
    },
}

impl ServerResponse {
    pub fn text(&self) -> &str {
        match self {
            Self::Tokens { text, .. } => text,
            Self::StreamEnd { .. } => panic!("Expected tokens"),
        }
    }

    pub fn message_num(&self) -> usize {
        match self {
            Self::StreamEnd { message_num, .. } | Self::Tokens { message_num, .. } => *message_num,
        }
    }

    pub fn is_stream_end(&self) -> bool {
        match self {
            Self::Tokens { .. } => false,
            Self::StreamEnd { .. } => true,
        }
    }
}
