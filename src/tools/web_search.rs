use log::debug;
use readability::extractor;
use serde::Deserialize;

use super::Tool;

pub struct WebSearch;

impl Tool for WebSearch {
    fn get_output(&self, input: &str) -> String {
        search(input);

        String::new()
    }

    fn name(&self) -> &str {
        "WEB_SEARCH"
    }
}

fn search(query: &str) {
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

    let scraped_html = scrape(&response.items.first().unwrap().link);

    debug!("Got scraped html:\n{scraped_html}");
}

fn scrape(url: &str) -> String {
    let r = extractor::scrape(url).expect("Could not scrape url");
    let text = r.text;

    debug!("Text: {text}");

    text
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
