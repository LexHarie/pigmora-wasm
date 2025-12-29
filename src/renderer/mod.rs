mod webgl;

use wasm_bindgen::JsValue;

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn is_valid(&self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }
}

pub struct Renderer {
    webgl: webgl::WebGlRenderer,
    width: u32,
    height: u32,
}

impl Renderer {
    pub fn new(canvas_id: &str) -> Result<Self, JsValue> {
        let webgl = webgl::WebGlRenderer::new(canvas_id)?;
        Ok(Self {
            webgl,
            width: 0,
            height: 0,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.webgl.resize(width, height);
    }

    pub fn render(&self, rects: &[Rect], selected: Option<Rect>) {
        if self.width == 0 || self.height == 0 {
            return;
        }

        self.webgl
            .render_scene(self.width, self.height, rects, selected);
    }
}
