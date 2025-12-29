use serde::{Deserialize, Serialize};

use super::Element;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layer {
    pub id: u32,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub elements: Vec<Element>,
}

impl Layer {
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            visible: true,
            locked: false,
            elements: Vec::new(),
        }
    }
}
