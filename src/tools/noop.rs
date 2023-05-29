use async_trait::async_trait;

use crate::model_client::ModelClient;

use super::Tool;

pub struct Noop;

#[async_trait]
impl Tool for Noop {
    async fn get_output(
        &self,
        _input: &str,
        _user_message: &str,
        _model_client: &(dyn ModelClient + Send + Sync),
    ) -> String {
        String::new()
    }

    fn name(&self) -> &str {
        "NONE"
    }
}
