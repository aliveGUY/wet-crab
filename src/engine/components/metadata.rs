use crate::index::engine::{ Component, ComponentUI, AttributeUI };

#[derive(Clone, Debug)]
pub struct Metadata {
    title: String,
}

impl Metadata {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}

impl Component for Metadata {
    fn to_ui(&self) -> ComponentUI {
        ComponentUI {
            name: "Metadata".into(),
            attributes: vec![AttributeUI {
                name: "title".into(),
                dt_type: "STRING".into(),
                value: "Meta title".into(),
            }],
        }
    }

    fn apply_ui(&mut self, component_ui: &ComponentUI) {
        // TODO: Implement UI application logic
    }
}
