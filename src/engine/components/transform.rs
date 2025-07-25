use crate::{
    index::engine::{
        utils::math::{ mat4x4_from_quat, mat4x4_mul, mat4x4_translate, Mat4x4 },
        Component,
        ComponentUI,
        AttributeUI,
    },
};

// Transform component for 3D objects - simplified matrix-based approach
#[derive(Clone, Debug)]
pub struct Transform {
    matrix: Mat4x4,
}

impl Transform {
    /// Create a new Transform with optional translation
    /// If no parameters provided, creates identity transform
    /// If x, y, z provided, creates transform with translation
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            matrix: mat4x4_translate(x, y, z),
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
    fn to_ui(&self) -> ComponentUI {
        ComponentUI {
            name: "Transform".into(),
            attributes: vec![
                AttributeUI {
                    name: "x positions".into(),
                    dt_type: "FLOAT".into(),
                    value: "0.0".into(),
                },
                AttributeUI {
                    name: "y positions".into(),
                    dt_type: "FLOAT".into(),
                    value: "0.0".into(),
                },
                AttributeUI {
                    name: "z positions".into(),
                    dt_type: "FLOAT".into(),
                    value: "0.0".into(),
                },
                AttributeUI {
                    name: "x rotation".into(),
                    dt_type: "INT".into(),
                    value: "0".into(),
                },
                AttributeUI {
                    name: "x rotation".into(),
                    dt_type: "INT".into(),
                    value: "0".into(),
                },
                AttributeUI {
                    name: "x rotation".into(),
                    dt_type: "INT".into(),
                    value: "0".into(),
                }
            ],
        }
    }

    fn apply_ui(&mut self, component_ui: &ComponentUI) {
        // TODO: Implement UI application logic
    }
}
