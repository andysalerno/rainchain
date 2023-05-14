






use crate::model_client::{ModelClient};

use super::Tool;

pub struct Noop;

impl Tool for Noop {
    fn get_output(
        &self,
        _input: &str,
        _user_message: &str,
        _model_client: &dyn ModelClient,
    ) -> String {
        String::new()
    }

    fn name(&self) -> &str {
        "NONE"
    }
}
