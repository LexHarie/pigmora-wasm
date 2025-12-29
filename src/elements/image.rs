use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageElement {
    pub source: Option<String>,
}

impl ImageElement {
    pub fn new() -> Self {
        Self { source: None }
    }
}
