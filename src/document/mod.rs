mod canvas;
mod element;
mod history;
mod layer;
mod transform;

use serde::{Deserialize, Serialize};

use crate::elements::{ElementData, ShapeElement};

pub use canvas::Canvas;
pub use element::{Element, ElementUpdate};
pub use history::{Command, History};
pub use layer::Layer;
pub use transform::Transform2D;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document {
    pub canvas: Canvas,
    pub layers: Vec<Layer>,
    pub active_layer_id: u32,
    next_id: u32,
}

impl Document {
    pub fn new(width: u32, height: u32) -> Self {
        let base_layer = Layer::new(1, "Layer 1");
        Self {
            canvas: Canvas::new(width, height),
            layers: vec![base_layer],
            active_layer_id: 1,
            next_id: 2,
        }
    }

    pub fn set_canvas_size(&mut self, width: u32, height: u32) {
        self.canvas.width = width;
        self.canvas.height = height;
    }

    pub fn next_element_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn recalculate_next_id(&mut self) {
        let mut max_id = 0;
        for layer in &self.layers {
            max_id = max_id.max(layer.id);
            for element in &layer.elements {
                max_id = max_id.max(element.id);
            }
        }
        self.next_id = max_id.saturating_add(1);
        if self.layers.is_empty() {
            let id = self.next_element_id();
            self.layers.push(Layer::new(id, "Layer 1"));
            self.active_layer_id = id;
        }
    }

    pub fn add_layer(&mut self, name: impl Into<String>) -> u32 {
        let id = self.next_element_id();
        self.layers.push(Layer::new(id, name));
        id
    }

    pub fn insert_element_at(&mut self, layer_id: u32, index: usize, element: Element) -> bool {
        if let Some(layer) = self.layers.iter_mut().find(|layer| layer.id == layer_id) {
            let insert_index = index.min(layer.elements.len());
            layer.elements.insert(insert_index, element);
            return true;
        }
        false
    }

    pub fn push_element(&mut self, layer_id: u32, element: Element) -> Option<usize> {
        if let Some(layer) = self.layers.iter_mut().find(|layer| layer.id == layer_id) {
            let index = layer.elements.len();
            layer.elements.push(element);
            return Some(index);
        }
        None
    }

    pub fn remove_element_by_id(&mut self, element_id: u32) -> Option<(u32, usize, Element)> {
        for layer in &mut self.layers {
            if let Some(index) = layer.elements.iter().position(|el| el.id == element_id) {
                let element = layer.elements.remove(index);
                return Some((layer.id, index, element));
            }
        }
        None
    }

    pub fn replace_element_by_id(&mut self, element_id: u32, element: Element) -> bool {
        for layer in &mut self.layers {
            if let Some(index) = layer.elements.iter().position(|el| el.id == element_id) {
                layer.elements[index] = element;
                return true;
            }
        }
        false
    }

    pub fn replace_element_at(
        &mut self,
        layer_id: u32,
        index: usize,
        element: Element,
    ) -> bool {
        if let Some(layer) = self.layers.iter_mut().find(|layer| layer.id == layer_id) {
            if index < layer.elements.len() && layer.elements[index].id == element.id {
                layer.elements[index] = element;
                return true;
            }
        }
        self.replace_element_by_id(element.id, element)
    }

    pub fn apply_update(
        &mut self,
        element_id: u32,
        update: &ElementUpdate,
    ) -> Option<(u32, usize, Element, Element)> {
        for layer in &mut self.layers {
            if let Some(index) = layer.elements.iter().position(|el| el.id == element_id) {
                let before = layer.elements[index].clone();
                let mut after = before.clone();
                update.apply_to(&mut after);
                layer.elements[index] = after.clone();
                return Some((layer.id, index, before, after));
            }
        }
        None
    }

    pub fn set_element_transform(&mut self, element_id: u32, transform: Transform2D) -> bool {
        for layer in &mut self.layers {
            if let Some(element) = layer.elements.iter_mut().find(|el| el.id == element_id) {
                element.transform = transform;
                return true;
            }
        }
        false
    }

    pub fn get_element_transform(&self, element_id: u32) -> Option<Transform2D> {
        for layer in &self.layers {
            if let Some(element) = layer.elements.iter().find(|el| el.id == element_id) {
                return Some(element.transform);
            }
        }
        None
    }

    pub fn find_first_shape(&self) -> Option<u32> {
        for layer in &self.layers {
            for element in &layer.elements {
                if matches!(element.data, ElementData::Shape(_)) {
                    return Some(element.id);
                }
            }
        }
        None
    }

    pub fn ensure_primary_shape(&mut self, transform: Transform2D) -> u32 {
        if let Some(element_id) = self.find_first_shape() {
            self.set_element_transform(element_id, transform);
            return element_id;
        }

        let element_id = self.next_element_id();
        let shape = ShapeElement::rectangle();
        let element = Element::shape(element_id, "Rectangle", shape, transform);
        let layer_id = self.active_layer_id;
        if self.push_element(layer_id, element).is_none() {
            let layer_id = self.add_layer("Layer 1");
            self.active_layer_id = layer_id;
            let fallback_element =
                Element::shape(element_id, "Rectangle", ShapeElement::rectangle(), transform);
            self.push_element(layer_id, fallback_element);
        }
        element_id
    }
}
