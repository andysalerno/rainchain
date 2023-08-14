use super::Tool;
use crate::model_client::ModelClient;
use async_trait::async_trait;

pub struct HomeAutomation;

#[async_trait]
impl Tool for HomeAutomation {
    fn name(&self) -> &str {
        "HOME_AUTOMATION"
    }

    async fn get_output(
        &self,
        input: &str,
        _user_message: &str,
        model_client: &(dyn ModelClient + Send + Sync),
    ) -> String {
        todo!("not done yet")
    }
}
