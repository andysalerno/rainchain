use crate::model_client::ModelClient;

pub mod web_search;

pub trait Tool {
    fn get_output(&self, input: &str, model_client: &dyn ModelClient) -> String;
    fn name(&self) -> &str;
}
