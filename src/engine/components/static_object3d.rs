// Import shared components
use crate::index::engine::{
    components::SharedComponents::{ Material, Mesh },
    Component,
};
use crate::{ ComponentUI, AttributeUI };
use slint::{ VecModel, ModelRc };
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]
pub struct StaticObject3D {
    pub mesh: Mesh,
    pub material: Material, // Required, no Option
    component_ui: Rc<RefCell<ComponentUI>>, // Single-threaded shared UI
}

impl StaticObject3D {
    pub fn new(mesh: Mesh, material: Material) -> Self {
        let attributes: Vec<AttributeUI> = Vec::new(); // Empty for now
        
        let component_ui = ComponentUI {
            name: "Static Object 3D".into(),
            attributes: ModelRc::new(VecModel::from(attributes)),
        };

        Self {
            mesh,
            material,
            component_ui: Rc::new(RefCell::new(component_ui)),
        }
    }

}

impl Component for StaticObject3D {
    fn apply_ui(&mut self, component_ui: &ComponentUI) {
        // Apply UI changes back to the component
        use slint::Model;
        
        for i in 0..component_ui.attributes.row_count() {
            if let Some(_attribute) = component_ui.attributes.row_data(i) {
                // TODO: Implement UI application logic when attributes are added
            }
        }
    }

    fn update_component_ui(&mut self, entity_id: &str) {
        // Update SharedStrings in the Rc<RefCell<ComponentUI>> with current component values
        let mut _ui = self.component_ui.borrow_mut();
        
        // TODO: Update SharedStrings when attributes are added
        
        println!("🔄 StaticObject3D SharedStrings updated for entity {}", entity_id);
    }

    fn get_component_ui(&self) -> Rc<RefCell<ComponentUI>> {
        self.component_ui.clone()
    }
}
