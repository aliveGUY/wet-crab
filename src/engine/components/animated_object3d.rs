// Import shared components
use crate::index::engine::{
    components::shared_components::{ Material, Mesh },
    Component,
};
use crate::{ ComponentUI, AttributeUI };
use slint::{ VecModel, ModelRc };
use std::rc::Rc;
use std::cell::RefCell;

// Import animation-specific components
mod skeleton_mod {
    include!("skeleton.rs");
}
mod animation_mod {
    include!("animation_state.rs");
}
mod animator_mod {
    include!("animator.rs");
}

pub use skeleton_mod::*;
pub use animation_mod::*;
pub use animator_mod::Animator;

#[derive(Clone)]
pub struct AnimatedObject3D {
    pub mesh: Mesh,
    pub material: Material, // Required, no Option
    pub skeleton: Skeleton, // Required, no Option
    pub animation_channels: Vec<AnimationChannel>, // Required
    pub animator: Animator, // Required, now public for system access
    component_ui: Rc<RefCell<ComponentUI>>, // Single-threaded shared UI
}

impl AnimatedObject3D {
    pub fn new(
        mesh: Mesh,
        material: Material,
        skeleton: Skeleton,
        animation_channels: Vec<AnimationChannel>
    ) -> Self {
        let attributes: Vec<AttributeUI> = Vec::new(); // Empty for now
        
        let component_ui = ComponentUI {
            name: "Animated Object 3D".into(),
            attributes: ModelRc::new(VecModel::from(attributes)),
        };

        Self {
            mesh,
            material,
            skeleton,
            animation_channels,
            animator: Animator::new(),
            component_ui: Rc::new(RefCell::new(component_ui)),
        }
    }

}

impl Component for AnimatedObject3D {
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
        
        println!("🔄 AnimatedObject3D SharedStrings updated for entity {}", entity_id);
    }

    fn get_component_ui(&self) -> Rc<RefCell<ComponentUI>> {
        self.component_ui.clone()
    }
}
