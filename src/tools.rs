use async_trait::async_trait;

use crate::model_client::ModelClient;

pub mod home_automation;
pub mod noop;
pub mod web_search;

#[async_trait]
pub trait Tool {
    async fn get_output(
        &self,
        input: &str,
        user_message: &str,
        model_client: &(dyn ModelClient + Send + Sync),
    ) -> String;

    fn name(&self) -> &str;
}
