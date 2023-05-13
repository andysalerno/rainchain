use std::vec;

use log::debug;
use ordered_float::OrderedFloat;
use readability::extractor;
use serde::Deserialize;

use crate::model_client::{Embedding, EmbeddingsRequest, ModelClient};

use super::Tool;

pub struct WebSearch;

impl Tool for WebSearch {
    fn get_output(&self, input: &str, model_client: &dyn ModelClient) -> String {
        let top_links = search(input);

        let sections: Vec<_> = top_links
            .into_iter()
            .take(1)
            .map(|url| scrape(&url))
            .flat_map(|text| split_text_into_sections(&text))
            .collect();

        debug!("Getting embeddings...");
        let embeddings_result = model_client.request_embeddings(&EmbeddingsRequest::new(sections));
        debug!("Got embeddings.");

        let mut corpus_embeddings = embeddings_result.take_embeddings();
        corpus_embeddings.sort_unstable_by_key(Embedding::index);

        let user_input_embedding = {
            let response =
                model_client.request_embeddings(&EmbeddingsRequest::new(vec![input.into()]));
            let embeddings = response.take_embeddings();
            embeddings.into_iter().next().expect("Expected embeddings")
        };

        let mut with_scores: Vec<_> = corpus_embeddings
            .into_iter()
            .map(|e| {
                let similarity = cosine_similarity(user_input_embedding.embedding(), e.embedding());
                (e, OrderedFloat(similarity))
            })
            .collect();

        with_scores.sort_unstable_by_key(|(_, score)| *score);

        String::new()
    }

    fn name(&self) -> &str {
        "WEB_SEARCH"
    }
}

fn split_text_into_sections(input: &str) -> Vec<String> {
    vec![input.into()]
}

fn cosine_similarity(vec1: &[f32], vec2: &[f32]) -> f32 {
    let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
    let magnitude_vec1: f32 = vec1.iter().map(|&n| n.powi(2)).sum::<f32>().sqrt();
    let magnitude_vec2: f32 = vec2.iter().map(|&n| n.powi(2)).sum::<f32>().sqrt();

    dot_product / (magnitude_vec1 * magnitude_vec2)
}

fn search(query: &str) -> Vec<String> {
    debug!("Searching Google for '{query}'");

    let (api_key, cx) = get_api_key_cx();
    let client = reqwest::blocking::Client::new();

    let response = client
        .get("https://www.googleapis.com/customsearch/v1")
        .query(&[("key", api_key.as_str()), ("cx", cx.as_str()), ("q", query)])
        .send()
        .unwrap()
        .json::<Response>()
        .unwrap();

    let len = response.items.len();
    debug!("Got {len} results");

    response.items.into_iter().map(|i| i.link).collect()
}

fn scrape(url: &str) -> String {
    let r = extractor::scrape(url).expect("Could not scrape url");

    r.text
}

fn get_api_key_cx() -> (String, String) {
    let api_key = std::fs::read_to_string("src/.googlekey.txt").unwrap();
    let cx = std::fs::read_to_string("src/.googlecx.txt").unwrap();

    (api_key, cx)
}

#[derive(Deserialize)]
struct Response {
    items: Vec<Item>,
}

#[derive(Deserialize)]
struct Item {
    title: String,
    link: String,
    snippet: String,
}
