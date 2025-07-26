use crate::index::engine::Component;
use crate::{ ComponentUI, AttributeUI };
use slint::{ VecModel, ModelRc };
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone, Debug)]
pub struct Metadata {
    title: String,
    component_ui: Rc<RefCell<ComponentUI>>, // Single-threaded shared UI
}

impl Metadata {
    pub fn new(title: &str) -> Self {
        let attributes = vec![AttributeUI {
            name: "title".into(),
            dt_type: "STRING".into(),
            value: title.into(),
        }];

        let component_ui = ComponentUI {
            name: "Metadata".into(),
            attributes: ModelRc::new(VecModel::from(attributes)),
        };

        Self {
            title: title.to_string(),
            component_ui: Rc::new(RefCell::new(component_ui)),
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}

impl Component for Metadata {
    fn apply_ui(&mut self, component_ui: &ComponentUI) {
        use slint::Model;
        
        for i in 0..component_ui.attributes.row_count() {
            if let Some(attribute) = component_ui.attributes.row_data(i) {
                if attribute.name.as_str() == "title" {
                    self.title = attribute.value.to_string();
                }
            }
        }
    }

    fn update_component_ui(&mut self, entity_id: &str) {
        // Update SharedStrings in the Rc<RefCell<ComponentUI>> with current component values
        let mut ui = self.component_ui.borrow_mut();
        
        // Update existing SharedStrings in-place
        use slint::Model;
        for i in 0..ui.attributes.row_count() {
            if let Some(mut attr) = ui.attributes.row_data(i) {
                match attr.name.as_str() {
                    "title" => attr.value = self.title.clone().into(),
                    _ => {}
                }
                ui.attributes.set_row_data(i, attr);
            }
        }
        
        println!("🔄 Metadata SharedStrings updated for entity {}", entity_id);
    }

    fn get_component_ui(&self) -> Rc<RefCell<ComponentUI>> {
        self.component_ui.clone()
    }
}
