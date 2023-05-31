use async_trait::async_trait;
use futures_util::Stream;

mod thought_action_agent;

pub use thought_action_agent::ThoughtActionAgent;

#[async_trait]
pub trait Agent {
    async fn get_response(&mut self, message: &str) -> String;
    async fn get_response_stream(
        &mut self,
        message: &str,
    ) -> Box<dyn Stream<Item = String> + Unpin + Send>;
}
