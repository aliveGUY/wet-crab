use crate::index::math::{Mat4x4, mat4x4_identity, mat4x4_translate, mat4x4_from_quat, mat4x4_mul};

// Transform component for 3D objects - simplified matrix-based approach
#[derive(Clone)]
pub struct Transform {
    matrix: Mat4x4,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            matrix: mat4x4_identity(),
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
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}
