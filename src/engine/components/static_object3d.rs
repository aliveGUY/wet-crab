// Import shared components
use crate::index::engine::{
    components::SharedComponents::{ Material, Mesh },
    Component,
    ComponentUI,
};

#[derive(Clone)]
pub struct StaticObject3D {
    pub mesh: Mesh,
    pub material: Material, // Required, no Option
}

impl StaticObject3D {
    pub fn new(mesh: Mesh, material: Material) -> Self {
        Self {
            mesh,
            material,
        }
    }
}

impl Component for StaticObject3D {
    fn to_ui(&self) -> ComponentUI {
        ComponentUI {
            name: "Static Object 3D".into(),
            attributes: Vec::new(),
        }
    }

    fn apply_ui(&mut self, component_ui: &ComponentUI) {
        // TODO: Implement UI application logic
    }
}
