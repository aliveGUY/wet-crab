use crate::index::engine::Component;
use crate::{ComponentUI, AttributeUI};
use slint::{VecModel, ModelRc};
use std::rc::Rc;
use std::cell::RefCell;
use serde::{Serialize, Deserialize};

pub type Vec3 = [f32; 3];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Shape {
    Sphere {
        radius: f32,
    },
    Capsule {
        radius: f32,
        height: f32,
    },
    Box {
        half_extents: Vec3,
    },
    Cylinder {
        radius: f32,
        height: f32,
    },
}

impl Shape {
    fn get_shape_name(&self) -> String {
        match self {
            Shape::Sphere { radius } => format!("Sphere (r: {:.2})", radius),
            Shape::Capsule { radius, height } => format!("Capsule (r: {:.2}, h: {:.2})", radius, height),
            Shape::Box { half_extents } => format!("Box ({:.2}, {:.2}, {:.2})", half_extents[0], half_extents[1], half_extents[2]),
            Shape::Cylinder { radius, height } => format!("Cylinder (r: {:.2}, h: {:.2})", radius, height),
        }
    }
}

impl Component for Shape {
    fn apply_ui(&mut self, _component_ui: &ComponentUI) {
        // Shape components are read-only for now
    }

    fn update_component_ui(&mut self, _entity_id: &str) {
        // Update UI if needed
    }

    fn get_component_ui(&self) -> Rc<RefCell<ComponentUI>> {
        let attributes = vec![
            AttributeUI {
                name: "Shape Type".into(),
                dt_type: "STRING".into(),
                value: self.get_shape_name().into(),
            },
        ];

        let component_ui = ComponentUI {
            name: "Shape".into(),
            attributes: ModelRc::new(VecModel::from(attributes)),
        };

        Rc::new(RefCell::new(component_ui))
    }
}
