use crate::model_client::ModelClient;

pub mod noop;
pub mod web_search;

pub trait Tool {
    fn get_output(&self, input: &str, user_message: &str, model_client: &dyn ModelClient)
        -> String;
    fn name(&self) -> &str;
}
