mod web_search;

pub trait Tool {
    fn get_output(&self, input: &str) -> String;
}
