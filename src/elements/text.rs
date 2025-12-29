use serde::{Deserialize, Serialize};

use crate::document::Color;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextElement {
    pub content: String,
    pub font_family: String,
    pub font_size: f32,
    pub fill: Color,
}

impl TextElement {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            font_family: "system-ui".to_string(),
            font_size: 24.0,
            fill: Color::new(0.1, 0.1, 0.1, 1.0),
        }
    }
}
