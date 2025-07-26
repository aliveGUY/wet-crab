use crate::{
    index::engine::{
        utils::math::{ 
            mat4x4_from_quat, mat4x4_mul, mat4x4_translate, mat4x4_scale, Mat4x4,
            mat4x4_extract_translation, mat4x4_extract_scale, mat4x4_extract_euler_angles,
            mat4x4_rot_x, mat4x4_rot_y, mat4x4_rot_z
        },
        Component,
    },
    ComponentUI,
    AttributeUI,
};
use slint::{ VecModel, ModelRc };
use std::rc::Rc;
use std::cell::RefCell;

// Transform component for 3D objects - simplified matrix-based approach
#[derive(Clone, Debug)]
pub struct Transform {
    matrix: Mat4x4,
    component_ui: Rc<RefCell<ComponentUI>>, // Single-threaded shared UI
}

impl Transform {
    /// Create a new Transform with optional translation
    /// If no parameters provided, creates identity transform
    /// If x, y, z provided, creates transform with translation
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        let attributes = vec![
            AttributeUI {
                name: "x position".into(),
                dt_type: "FLOAT".into(),
                value: format!("{:.3}", x).into(),
            },
            AttributeUI {
                name: "y position".into(),
                dt_type: "FLOAT".into(),
                value: "0.0".into(),
            },
            AttributeUI {
                name: "z position".into(),
                dt_type: "FLOAT".into(),
                value: format!("{:.3}", z).into(),
            },
            AttributeUI {
                name: "x scale".into(),
                dt_type: "FLOAT".into(),
                value: "1.0".into(),
            },
            AttributeUI {
                name: "y scale".into(),
                dt_type: "FLOAT".into(),
                value: "1.0".into(),
            },
            AttributeUI {
                name: "z scale".into(),
                dt_type: "FLOAT".into(),
                value: "1.0".into(),
            },
            AttributeUI {
                name: "pitch (degrees)".into(),
                dt_type: "FLOAT".into(),
                value: "0.0".into(),
            },
            AttributeUI {
                name: "yaw (degrees)".into(),
                dt_type: "FLOAT".into(),
                value: "0.0".into(),
            },
            AttributeUI {
                name: "roll (degrees)".into(),
                dt_type: "FLOAT".into(),
                value: "0.0".into(),
            }
        ];

        let component_ui = ComponentUI {
            name: "Transform".into(),
            attributes: ModelRc::new(VecModel::from(attributes)),
        };

        Self {
            matrix: mat4x4_translate(x, y, z),
            component_ui: Rc::new(RefCell::new(component_ui)),
        }
    }

    /// Apply translation to the transform
    /// Receives new position coordinates
    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        let translation_matrix = mat4x4_translate(x, y, z);
        self.matrix = mat4x4_mul(translation_matrix, self.matrix);
    }

    /// Apply rotation to the transform
    /// Receives quaternion components (x, y, z, w)
    pub fn rotate(&mut self, x: f32, y: f32, z: f32, w: f32) {
        let rotation_matrix = mat4x4_from_quat([x, y, z, w]);
        self.matrix = mat4x4_mul(rotation_matrix, self.matrix);
    }

    /// Get the transformation matrix
    pub fn get_matrix(&self) -> &Mat4x4 {
        &self.matrix
    }

    /// Get mutable reference to the transformation matrix
    pub fn get_matrix_mut(&mut self) -> &mut Mat4x4 {
        &mut self.matrix
    }
}

impl Component for Transform {
    fn apply_ui(&mut self, component_ui: &ComponentUI) {
        // Extract current values
        let mut translation = mat4x4_extract_translation(&self.matrix);
        let mut scale = mat4x4_extract_scale(&self.matrix);
        let mut euler_angles = mat4x4_extract_euler_angles(&self.matrix);
        
        // Apply UI changes
        use slint::Model;
        
        for i in 0..component_ui.attributes.row_count() {
            if let Some(attribute) = component_ui.attributes.row_data(i) {
            match attribute.name.as_str() {
                "x position" => {
                    if let Ok(value) = attribute.value.parse::<f32>() {
                        translation[0] = value;
                    }
                },
                "y position" => {
                    if let Ok(value) = attribute.value.parse::<f32>() {
                        translation[1] = value;
                    }
                },
                "z position" => {
                    if let Ok(value) = attribute.value.parse::<f32>() {
                        translation[2] = value;
                    }
                },
                "x scale" => {
                    if let Ok(value) = attribute.value.parse::<f32>() {
                        scale[0] = value;
                    }
                },
                "y scale" => {
                    if let Ok(value) = attribute.value.parse::<f32>() {
                        scale[1] = value;
                    }
                },
                "z scale" => {
                    if let Ok(value) = attribute.value.parse::<f32>() {
                        scale[2] = value;
                    }
                },
                "pitch (degrees)" => {
                    if let Ok(degrees) = attribute.value.parse::<f32>() {
                        euler_angles[0] = degrees.to_radians();
                    }
                },
                "yaw (degrees)" => {
                    if let Ok(degrees) = attribute.value.parse::<f32>() {
                        euler_angles[1] = degrees.to_radians();
                    }
                },
                "roll (degrees)" => {
                    if let Ok(degrees) = attribute.value.parse::<f32>() {
                        euler_angles[2] = degrees.to_radians();
                    }
                },
                _ => {}
            }
            }
        }
        
        // Rebuild the transformation matrix from the modified components
        // Order: Scale -> Rotate -> Translate
        let scale_matrix = mat4x4_scale(scale[0], scale[1], scale[2]);
        let rotation_x = mat4x4_rot_x(euler_angles[0]); // pitch
        let rotation_y = mat4x4_rot_y(euler_angles[1]); // yaw
        let rotation_z = mat4x4_rot_z(euler_angles[2]); // roll
        let translation_matrix = mat4x4_translate(translation[0], translation[1], translation[2]);
        
        // Combine transformations: T * R * S
        let rotation_matrix = mat4x4_mul(mat4x4_mul(rotation_y, rotation_x), rotation_z);
        let transform_matrix = mat4x4_mul(rotation_matrix, scale_matrix);
        self.matrix = mat4x4_mul(translation_matrix, transform_matrix);
    }

    fn update_component_ui(&mut self, entity_id: &str) {
        // Update SharedStrings in the Rc<RefCell<ComponentUI>> with current component values
        let mut ui = self.component_ui.borrow_mut();
        
        // Extract actual values from the transformation matrix
        let translation = mat4x4_extract_translation(&self.matrix);
        let scale = mat4x4_extract_scale(&self.matrix);
        let euler_angles = mat4x4_extract_euler_angles(&self.matrix);
        
        // Convert radians to degrees for better UI display
        let pitch_degrees = euler_angles[0].to_degrees();
        let yaw_degrees = euler_angles[1].to_degrees();
        let roll_degrees = euler_angles[2].to_degrees();

        // Update existing SharedStrings in-place
        use slint::Model;
        for i in 0..ui.attributes.row_count() {
            if let Some(mut attr) = ui.attributes.row_data(i) {
                match attr.name.as_str() {
                    "x position" => attr.value = format!("{:.3}", translation[0]).into(),
                    "y position" => attr.value = format!("{:.3}", translation[1]).into(),
                    "z position" => attr.value = format!("{:.3}", translation[2]).into(),
                    "x scale" => attr.value = format!("{:.3}", scale[0]).into(),
                    "y scale" => attr.value = format!("{:.3}", scale[1]).into(),
                    "z scale" => attr.value = format!("{:.3}", scale[2]).into(),
                    "pitch (degrees)" => attr.value = format!("{:.1}", pitch_degrees).into(),
                    "yaw (degrees)" => attr.value = format!("{:.1}", yaw_degrees).into(),
                    "roll (degrees)" => attr.value = format!("{:.1}", roll_degrees).into(),
                    _ => {}
                }
                ui.attributes.set_row_data(i, attr);
            }
        }
        
        println!("🔄 Transform SharedStrings updated for entity {}", entity_id);
    }

    fn get_component_ui(&self) -> Rc<RefCell<ComponentUI>> {
        self.component_ui.clone()
    }
}
