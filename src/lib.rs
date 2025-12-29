mod document;
mod elements;
mod renderer;

use document::{Command, Document, ElementUpdate, History, Transform2D};
use elements::{ImageElement, ShapeElement, ShapeType, TextElement};
use renderer::{Rect, Renderer};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct PigmoraEngine {
    renderer: Renderer,
    document: Document,
    history: History,
    primary_element_id: Option<u32>,
}

#[wasm_bindgen]
impl PigmoraEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<PigmoraEngine, JsValue> {
        console_error_panic_hook::set_once();
        let renderer = Renderer::new(canvas_id)?;
        Ok(PigmoraEngine {
            renderer,
            document: Document::new(0, 0),
            history: History::new(),
            primary_element_id: None,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
        self.document.set_canvas_size(width, height);
    }

    pub fn set_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let transform = Transform2D::new(x, y, width, height);
        let element_id = match self.primary_element_id {
            Some(element_id) => {
                self.document
                    .set_element_transform(element_id, transform);
                element_id
            }
            None => {
                let element_id = self.document.ensure_primary_shape(transform);
                self.primary_element_id = Some(element_id);
                element_id
            }
        };
        self.primary_element_id = Some(element_id);
    }

    pub fn render(&mut self) {
        let (rects, selected) = self.collect_rects();
        self.renderer.render(&rects, selected);
    }

    pub fn get_document(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.document)
            .map_err(|err| JsValue::from_str(&err.to_string()))
    }

    pub fn load_document(&mut self, value: JsValue) -> Result<(), JsValue> {
        let document: Document = serde_wasm_bindgen::from_value(value)
            .map_err(|err| JsValue::from_str(&err.to_string()))?;
        self.document = document;
        self.document.recalculate_next_id();
        self.history.clear();
        self.primary_element_id = self.document.find_first_shape();
        self.sync_renderer_rect();
        Ok(())
    }

    pub fn undo(&mut self) -> bool {
        let changed = self.history.undo(&mut self.document);
        if changed {
            self.sync_renderer_rect();
        }
        changed
    }

    pub fn redo(&mut self) -> bool {
        let changed = self.history.redo(&mut self.document);
        if changed {
            self.sync_renderer_rect();
        }
        changed
    }

    pub fn add_shape(&mut self, shape_type: &str, x: f32, y: f32) -> Result<u32, JsValue> {
        let shape_type = parse_shape_type(shape_type)?;
        let transform = Transform2D::new(x, y, 160.0, 120.0);
        let element_id = self.document.next_element_id();
        let shape = ShapeElement {
            shape_type,
            ..ShapeElement::rectangle()
        };
        let element = document::Element::shape(element_id, "Shape", shape, transform);
        let layer_id = self.document.active_layer_id;
        let index = self
            .document
            .push_element(layer_id, element.clone())
            .ok_or_else(|| JsValue::from_str("Layer not found"))?;
        self.history.record(Command::AddElement {
            layer_id,
            index,
            element,
        });
        self.primary_element_id = Some(element_id);
        self.sync_renderer_rect();
        Ok(element_id)
    }

    pub fn add_text(&mut self, content: &str, x: f32, y: f32) -> Result<u32, JsValue> {
        let transform = Transform2D::new(x, y, 240.0, 80.0);
        let element_id = self.document.next_element_id();
        let text = TextElement::new(content);
        let element = document::Element::text(element_id, "Text", text, transform);
        let layer_id = self.document.active_layer_id;
        let index = self
            .document
            .push_element(layer_id, element.clone())
            .ok_or_else(|| JsValue::from_str("Layer not found"))?;
        self.history.record(Command::AddElement {
            layer_id,
            index,
            element,
        });
        self.primary_element_id = Some(element_id);
        self.sync_renderer_rect();
        Ok(element_id)
    }

    pub fn add_image(&mut self, _data: &[u8], x: f32, y: f32) -> Result<u32, JsValue> {
        let transform = Transform2D::new(x, y, 320.0, 200.0);
        let element_id = self.document.next_element_id();
        let image = ImageElement::new();
        let element = document::Element::image(element_id, "Image", image, transform);
        let layer_id = self.document.active_layer_id;
        let index = self
            .document
            .push_element(layer_id, element.clone())
            .ok_or_else(|| JsValue::from_str("Layer not found"))?;
        self.history.record(Command::AddElement {
            layer_id,
            index,
            element,
        });
        self.primary_element_id = Some(element_id);
        self.sync_renderer_rect();
        Ok(element_id)
    }

    pub fn delete_element(&mut self, element_id: u32) -> bool {
        if let Some((layer_id, index, element)) = self.document.remove_element_by_id(element_id) {
            self.history.record(Command::DeleteElement {
                layer_id,
                index,
                element,
            });
            if self.primary_element_id == Some(element_id) {
                self.primary_element_id = self.document.find_first_shape();
            }
            self.sync_renderer_rect();
            return true;
        }
        false
    }

    pub fn update_element(&mut self, element_id: u32, props: JsValue) -> Result<bool, JsValue> {
        let update: ElementUpdate = serde_wasm_bindgen::from_value(props)
            .map_err(|err| JsValue::from_str(&err.to_string()))?;
        if let Some((layer_id, index, before, after)) =
            self.document.apply_update(element_id, &update)
        {
            self.history.record(Command::UpdateElement {
                layer_id,
                index,
                before,
                after,
            });
            if self.primary_element_id == Some(element_id) {
                self.sync_renderer_rect();
            }
            return Ok(true);
        }
        Ok(false)
    }
}

fn parse_shape_type(shape_type: &str) -> Result<ShapeType, JsValue> {
    match shape_type {
        "rect" | "rectangle" => Ok(ShapeType::Rect),
        "ellipse" => Ok(ShapeType::Ellipse),
        "line" => Ok(ShapeType::Line),
        "polygon" => Ok(ShapeType::Polygon),
        _ => Err(JsValue::from_str("Unknown shape type")),
    }
}

impl PigmoraEngine {
    fn collect_rects(&self) -> (Vec<Rect>, Option<Rect>) {
        let mut rects = Vec::new();
        let mut selected_rect = None;
        let selected_id = self
            .primary_element_id
            .or_else(|| self.document.find_first_shape());

        for layer in &self.document.layers {
            if !layer.visible {
                continue;
            }
            for element in &layer.elements {
                let transform = element.transform;
                let rect = Rect {
                    x: transform.x,
                    y: transform.y,
                    width: transform.width,
                    height: transform.height,
                };
                rects.push(rect);
                if Some(element.id) == selected_id {
                    selected_rect = Some(rect);
                }
            }
        }

        (rects, selected_rect)
    }

    fn sync_renderer_rect(&mut self) {
        let element_id = self
            .primary_element_id
            .or_else(|| self.document.find_first_shape());
        if let Some(element_id) = element_id {
            if self.document.get_element_transform(element_id).is_some() {
                self.primary_element_id = Some(element_id);
            }
        }
    }
}
