use serde::{Deserialize, Serialize};

use crate::elements::{ElementData, ImageElement, ShapeElement, TextElement};

use super::Transform2D;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Element {
    pub id: u32,
    pub name: String,
    pub transform: Transform2D,
    pub data: ElementData,
}

impl Element {
    pub fn new(id: u32, name: impl Into<String>, transform: Transform2D, data: ElementData) -> Self {
        Self {
            id,
            name: name.into(),
            transform,
            data,
        }
    }

    pub fn shape(id: u32, name: impl Into<String>, shape: ShapeElement, transform: Transform2D) -> Self {
        Self::new(id, name, transform, ElementData::Shape(shape))
    }

    pub fn text(id: u32, name: impl Into<String>, text: TextElement, transform: Transform2D) -> Self {
        Self::new(id, name, transform, ElementData::Text(text))
    }

    pub fn image(id: u32, name: impl Into<String>, image: ImageElement, transform: Transform2D) -> Self {
        Self::new(id, name, transform, ElementData::Image(image))
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ElementUpdate {
    pub name: Option<String>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub rotation: Option<f32>,
}

impl ElementUpdate {
    pub fn apply_to(&self, element: &mut Element) {
        if let Some(name) = &self.name {
            element.name = name.clone();
        }

        if let Some(x) = self.x {
            element.transform.x = x;
        }
        if let Some(y) = self.y {
            element.transform.y = y;
        }
        if let Some(width) = self.width {
            element.transform.width = width.max(1.0);
        }
        if let Some(height) = self.height {
            element.transform.height = height.max(1.0);
        }
        if let Some(rotation) = self.rotation {
            element.transform.rotation = rotation;
        }
    }
}
