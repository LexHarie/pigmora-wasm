use crate::document::{Document, Element};

#[derive(Clone, Debug)]
pub enum Command {
    AddElement {
        layer_id: u32,
        index: usize,
        element: Element,
    },
    DeleteElement {
        layer_id: u32,
        index: usize,
        element: Element,
    },
    UpdateElement {
        layer_id: u32,
        index: usize,
        before: Element,
        after: Element,
    },
}

impl Command {
    fn apply(&self, document: &mut Document) -> bool {
        match self {
            Command::AddElement {
                layer_id,
                index,
                element,
            } => document.insert_element_at(*layer_id, *index, element.clone()),
            Command::DeleteElement { element, .. } => {
                document.remove_element_by_id(element.id).is_some()
            }
            Command::UpdateElement {
                layer_id,
                index,
                after,
                ..
            } => document.replace_element_at(*layer_id, *index, after.clone()),
        }
    }

    fn undo(&self, document: &mut Document) -> bool {
        match self {
            Command::AddElement { element, .. } => {
                document.remove_element_by_id(element.id).is_some()
            }
            Command::DeleteElement {
                layer_id,
                index,
                element,
            } => document.insert_element_at(*layer_id, *index, element.clone()),
            Command::UpdateElement {
                layer_id,
                index,
                before,
                ..
            } => document.replace_element_at(*layer_id, *index, before.clone()),
        }
    }
}

pub struct History {
    undo_stack: Vec<Command>,
    redo_stack: Vec<Command>,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    pub fn record(&mut self, command: Command) {
        self.undo_stack.push(command);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, document: &mut Document) -> bool {
        if let Some(command) = self.undo_stack.pop() {
            if command.undo(document) {
                self.redo_stack.push(command);
                return true;
            }
        }
        false
    }

    pub fn redo(&mut self, document: &mut Document) -> bool {
        if let Some(command) = self.redo_stack.pop() {
            if command.apply(document) {
                self.undo_stack.push(command);
                return true;
            }
        }
        false
    }
}
