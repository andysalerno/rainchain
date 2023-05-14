use std::{error::Error, vec};

use log::{debug, trace};
use ordered_float::OrderedFloat;

use serde::Deserialize;

use crate::model_client::{Embedding, EmbeddingsRequest, ModelClient};

use super::Tool;

pub struct Noop;

impl Tool for Noop {
    fn get_output(
        &self,
        input: &str,
        user_message: &str,
        model_client: &dyn ModelClient,
    ) -> String {
        String::new()
    }

    fn name(&self) -> &str {
        "NONE"
    }
}
