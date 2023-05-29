use std::{error::Error, time::Duration, vec};

use async_trait::async_trait;
use futures::future::{self, try_join_all};
use log::{debug, trace};
use ordered_float::OrderedFloat;
use serde::Deserialize;

use crate::model_client::{Embedding, EmbeddingsRequest, ModelClient};

use super::Tool;

const MAX_SECTION_LEN: usize = 400;
const TOP_N_SECTIONS: usize = 3;

pub struct WebSearch;

#[async_trait]
impl Tool for WebSearch {
    async fn get_output(
        &self,
        input: &str,
        user_message: &str,
        model_client: &(dyn ModelClient + Send + Sync),
    ) -> String {
        let top_links = search(input).await;

        let scrape_futures = top_links.into_iter().take(6).map(scrape);

        let sections: Vec<_> = future::join_all(scrape_futures)
            .await
            .into_iter()
            .filter_map(Result::ok)
            .filter(|text| text.len() > 50)
            .flat_map(|text| split_text_into_sections(text, MAX_SECTION_LEN))
            .collect();

        debug!("Getting embeddings...");
        let embeddings_result = model_client
            .request_embeddings(&EmbeddingsRequest::new(sections.clone()))
            .await;

        let mut corpus_embeddings = embeddings_result.take_embeddings();
        corpus_embeddings.sort_unstable_by_key(Embedding::index);

        {
            let len = corpus_embeddings.len();
            debug!("Got {len} embeddings.");
        }

        let user_embed_str: String = input.into();

        let user_input_embedding = {
            let response = model_client
                .request_embeddings(&EmbeddingsRequest::new(vec![user_embed_str.clone()]))
                .await;
            let embeddings = response.take_embeddings();
            embeddings.into_iter().next().expect("Expected embeddings")
        };

        debug!("Finding closest matches for: {user_embed_str}");
        let mut with_scores: Vec<_> = corpus_embeddings
            .into_iter()
            .map(|e| {
                let similarity = cosine_similarity(user_input_embedding.embedding(), e.embedding());
                (e, OrderedFloat(similarity))
            })
            .collect();

        with_scores.sort_unstable_by_key(|(_, score)| -*score);

        let mut result = String::new();
        for (n, (embedding, score)) in with_scores.into_iter().take(TOP_N_SECTIONS).enumerate() {
            let index = embedding.index();
            let original_text = &sections[index];
            debug!("Score {score}: {original_text}");

            result.push_str(&format!("    [WEB_RESULT {n}]: {original_text}\n"));
        }

        // Trailing newline
        result.pop();

        result
    }

    fn name(&self) -> &str {
        "WEB_SEARCH"
    }
}

fn split_text_into_sections(input: impl Into<String>, max_section_len: usize) -> Vec<String> {
    let mut result = Vec::<String>::new();

    let input: String = input.into();

    for sentence in input
        .split_terminator(&['.', '\n'])
        .filter(|s| !s.is_empty())
    {
        let sentence = sentence.trim().to_owned();
        if let Some(last) = result.last_mut() {
            if last.len() + sentence.len() > max_section_len {
                result.push(sentence);
            } else {
                last.push_str(". ");
                last.push_str(&sentence);
            }
        } else {
            result.push(sentence);
        }
    }

    result
}

fn cosine_similarity(vec1: &[f32], vec2: &[f32]) -> f32 {
    let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
    let magnitude_vec1: f32 = vec1.iter().map(|&n| n.powi(2)).sum::<f32>().sqrt();
    let magnitude_vec2: f32 = vec2.iter().map(|&n| n.powi(2)).sum::<f32>().sqrt();

    dot_product / (magnitude_vec1 * magnitude_vec2)
}

async fn search(query: &str) -> Vec<String> {
    debug!("Searching Google for '{query}'");

    let (api_key, cx) = get_api_key_cx();
    let client = reqwest::Client::new();

    let response = client
        .get("https://www.googleapis.com/customsearch/v1")
        .query(&[("key", api_key.as_str()), ("cx", cx.as_str()), ("q", query)])
        .timeout(Duration::from_millis(5000))
        .send()
        .await
        .unwrap()
        .json::<Response>()
        .await
        .unwrap();

    let len = response.items.len();
    debug!("Got {len} results");

    response.items.into_iter().map(|i| i.link).collect()
}

async fn scrape(url: impl AsRef<str>) -> Result<String, Box<dyn Error + Send + Sync>> {
    let url = url.as_ref();

    debug!("Scraping: {url}...");

    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_millis(500))
        .build()?;

    let response = client.get(url).send().await?;
    let s = response.text().await?;
    let mut readability = readable_readability::Readability::new();
    let (node_ref, _metadata) = readability
        .strip_unlikelys(true)
        .clean_attributes(true)
        .parse(&s);

    debug!("Done.");

    let text_content = node_ref.text_contents();

    trace!("Scraped text:\n{text_content}");

    Ok(text_content.trim().into())
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
