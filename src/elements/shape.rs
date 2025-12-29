use serde::{Deserialize, Serialize};

use crate::document::Color;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ShapeType {
    Rect,
    Ellipse,
    Line,
    Polygon,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Fill {
    pub color: Color,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Stroke {
    pub color: Color,
    pub width: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShapeElement {
    pub shape_type: ShapeType,
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
}

impl ShapeElement {
    pub fn rectangle() -> Self {
        Self {
            shape_type: ShapeType::Rect,
            fill: Some(Fill {
                color: Color::new(0.86, 0.42, 0.25, 1.0),
            }),
            stroke: None,
        }
    }
}
