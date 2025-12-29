pub mod image;
pub mod shape;
pub mod text;

use serde::{Deserialize, Serialize};

pub use image::ImageElement;
#[allow(unused_imports)]
pub use shape::{Fill, ShapeElement, ShapeType, Stroke};
pub use text::TextElement;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ElementData {
    Shape(ShapeElement),
    Text(TextElement),
    Image(ImageElement),
}
