use serde::{Deserialize, Serialize};

use crate::document::Color;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub background: Color,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            background: Color::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}
