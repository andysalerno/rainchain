use std::vec;

use log::debug;
use readability::extractor;
use serde::Deserialize;

use crate::model_client::{EmbeddingsRequest, ModelClient};

use super::Tool;

pub struct WebSearch;

impl Tool for WebSearch {
    fn get_output(&self, input: &str, model_client: &dyn ModelClient) -> String {
        let top_links = search(input);

        let x: Vec<_> = top_links
            .into_iter()
            .take(1)
            .map(|url| scrape(&url))
            .flat_map(|text| split_text_into_sections(&text))
            .collect();

        let embeddings =
            model_client.request_embeddings(&EmbeddingsRequest::new(x.first().unwrap().into()));

        debug!("Got embeddings:\n{embeddings:?}");

        String::new()
    }

    fn name(&self) -> &str {
        "WEB_SEARCH"
    }
}

fn split_text_into_sections(input: &str) -> Vec<String> {
    vec![input.into()]
}

fn get_embeddings(input: &[String]) {
    let first = input.first().unwrap();

    let request = EmbeddingsRequest::new(first.into());
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
