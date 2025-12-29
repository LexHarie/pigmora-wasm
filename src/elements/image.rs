use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ImageFilters {
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
}

impl Default for ImageFilters {
    fn default() -> Self {
        Self {
            brightness: 1.0,
            contrast: 1.0,
            saturation: 1.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageElement {
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub filters: ImageFilters,
}

impl ImageElement {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            filters: ImageFilters::default(),
        }
    }
}
