mod document;
mod elements;
mod renderer;

use document::{Command, Document, Element, ElementUpdate, History, Transform2D};
use elements::{ElementData, ImageElement, ShapeElement, ShapeType, TextElement};
use renderer::{Rect, RenderShape, Renderer, ShapeKind};
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy, Debug)]
enum Tool {
    Select,
    Shape,
    Text,
    Image,
}

#[derive(Clone, Debug)]
struct TransformSnapshot {
    element_id: u32,
    before: Element,
}

#[wasm_bindgen]
pub struct PigmoraEngine {
    renderer: Renderer,
    document: Document,
    history: History,
    selected_element_id: Option<u32>,
    active_tool: Tool,
    active_shape_type: ShapeType,
    transform_snapshot: Option<TransformSnapshot>,
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
            selected_element_id: None,
            active_tool: Tool::Select,
            active_shape_type: ShapeType::Rect,
            transform_snapshot: None,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
        self.document.set_canvas_size(width, height);
    }

    pub fn set_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let transform = Transform2D::new(x, y, width, height);
        let element_id = match self.selected_element_id {
            Some(element_id) => {
                self.document
                    .set_element_transform(element_id, transform);
                element_id
            }
            None => {
                let element_id = self.document.ensure_primary_shape(transform);
                self.selected_element_id = Some(element_id);
                element_id
            }
        };
        self.selected_element_id = Some(element_id);
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
        self.selected_element_id = self.document.find_first_shape();
        self.sync_selection();
        Ok(())
    }

    pub fn undo(&mut self) -> bool {
        let changed = self.history.undo(&mut self.document);
        if changed {
            self.sync_selection();
        }
        changed
    }

    pub fn redo(&mut self) -> bool {
        let changed = self.history.redo(&mut self.document);
        if changed {
            self.sync_selection();
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
        self.selected_element_id = Some(element_id);
        self.sync_selection();
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
        self.selected_element_id = Some(element_id);
        self.sync_selection();
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
        self.selected_element_id = Some(element_id);
        self.sync_selection();
        Ok(element_id)
    }

    pub fn delete_element(&mut self, element_id: u32) -> bool {
        if let Some((layer_id, index, element)) = self.document.remove_element_by_id(element_id) {
            self.history.record(Command::DeleteElement {
                layer_id,
                index,
                element,
            });
            if self.selected_element_id == Some(element_id) {
                self.selected_element_id = self.document.find_first_shape();
            }
            self.sync_selection();
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
            if self.selected_element_id == Some(element_id) {
                self.sync_selection();
            }
            return Ok(true);
        }
        Ok(false)
    }

    pub fn get_selected_id(&self) -> Option<u32> {
        self.selected_element_id
    }

    pub fn set_active_tool(&mut self, tool: &str) -> Result<(), JsValue> {
        self.active_tool = match tool {
            "select" => Tool::Select,
            "shape" => Tool::Shape,
            "text" => Tool::Text,
            "image" => Tool::Image,
            _ => return Err(JsValue::from_str("Unknown tool")),
        };
        Ok(())
    }

    pub fn set_active_shape(&mut self, shape_type: &str) -> Result<(), JsValue> {
        self.active_shape_type = parse_shape_type(shape_type)?;
        Ok(())
    }

    pub fn select_at(&mut self, x: f32, y: f32) -> Option<u32> {
        let hit = self.document.hit_test(x, y);
        self.selected_element_id = hit;
        hit
    }

    pub fn select_element(&mut self, element_id: u32) -> bool {
        if self.document.get_element_by_id(element_id).is_some() {
            self.selected_element_id = Some(element_id);
            return true;
        }
        false
    }

    pub fn begin_transform(&mut self) -> bool {
        let element_id = match self.selected_element_id {
            Some(element_id) => element_id,
            None => return false,
        };
        let element = match self.document.get_element_by_id(element_id) {
            Some(element) => element.clone(),
            None => return false,
        };
        self.transform_snapshot = Some(TransformSnapshot { element_id, before: element });
        true
    }

    pub fn update_selected_transform(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> bool {
        let element_id = match self.selected_element_id {
            Some(element_id) => element_id,
            None => return false,
        };
        if let Some(element) = self.document.get_element_by_id_mut(element_id) {
            element.transform.x = x;
            element.transform.y = y;
            element.transform.width = width.max(1.0);
            element.transform.height = height.max(1.0);
            return true;
        }
        false
    }

    pub fn update_selected_text_size(&mut self, font_size: f32) -> bool {
        let element_id = match self.selected_element_id {
            Some(element_id) => element_id,
            None => return false,
        };
        if let Some(element) = self.document.get_element_by_id_mut(element_id) {
            if let ElementData::Text(text) = &mut element.data {
                text.font_size = font_size.max(1.0);
                return true;
            }
        }
        false
    }

    pub fn commit_transform(&mut self) -> bool {
        let snapshot = match self.transform_snapshot.take() {
            Some(snapshot) => snapshot,
            None => return false,
        };
        let after = match self.document.get_element_by_id(snapshot.element_id) {
            Some(element) => element.clone(),
            None => return false,
        };
        if snapshot.before.transform == after.transform {
            return false;
        }
        if let Some((layer_id, index)) = self.document.find_element_location(snapshot.element_id) {
            self.history.record(Command::UpdateElement {
                layer_id,
                index,
                before: snapshot.before,
                after,
            });
            return true;
        }
        false
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
    fn collect_rects(&self) -> (Vec<RenderShape>, Option<Rect>) {
        let mut rects = Vec::new();
        let mut selected_rect = None;
        let selected_id = self.selected_element_id;

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
                if let ElementData::Shape(shape) = &element.data {
                    let shape_kind = match shape.shape_type {
                        ShapeType::Rect => ShapeKind::Rect,
                        ShapeType::Ellipse => ShapeKind::Ellipse,
                        ShapeType::Polygon => ShapeKind::Diamond,
                        ShapeType::Line => ShapeKind::Rect,
                    };
                    rects.push(RenderShape {
                        rect,
                        shape: shape_kind,
                    });
                } else if matches!(element.data, ElementData::Image(_)) {
                    rects.push(RenderShape {
                        rect,
                        shape: ShapeKind::Rect,
                    });
                }
                if Some(element.id) == selected_id {
                    selected_rect = Some(rect);
                }
            }
        }

        (rects, selected_rect)
    }

    fn sync_selection(&mut self) {
        let element_id = match self.selected_element_id {
            Some(element_id) => element_id,
            None => return,
        };
        if self.document.get_element_transform(element_id).is_none() {
            self.selected_element_id = self.document.find_first_shape();
        }
    }
}
