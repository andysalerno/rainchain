use super::Tool;

pub struct WebSearch;

impl Tool for WebSearch {
    fn get_output(&self, _input: &str) -> String {
        todo!()
    }

    fn name(&self) -> &str {
        "WEB_SEARCH"
    }
}
