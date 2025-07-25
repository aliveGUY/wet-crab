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
                value: self.title.clone().into(),
            }],
        }
    }

    fn apply_ui(&mut self, component_ui: &ComponentUI) {
        // Apply UI changes back to the component
        for attribute in &component_ui.attributes {
            if attribute.name.as_str() == "title" {
                self.title = attribute.value.to_string();
            }
        }
    }
}
