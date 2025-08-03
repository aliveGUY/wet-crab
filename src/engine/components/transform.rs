use crate::index::engine::utils::math::{
    mat4x4_from_quat, mat4x4_mul, mat4x4_translate, mat4x4_scale, Mat4x4,
    mat4x4_extract_translation, mat4x4_extract_scale, mat4x4_extract_euler_angles,
    mat4x4_rot_x, mat4x4_rot_y, mat4x4_rot_z
};
use serde::{Serialize, Deserialize};

// Transform component for 3D objects - simplified matrix-based approach
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transform {
    pub matrix: Mat4x4,
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

    /// Set the matrix from position, rotation (euler angles in radians), and scale
    pub fn set_from_components(&mut self, position: [f32; 3], rotation: [f32; 3], scale: [f32; 3]) {
        // Order: Scale -> Rotate -> Translate
        let scale_matrix = mat4x4_scale(scale[0], scale[1], scale[2]);
        let rotation_x = mat4x4_rot_x(rotation[0]); // pitch
        let rotation_y = mat4x4_rot_y(rotation[1]); // yaw
        let rotation_z = mat4x4_rot_z(rotation[2]); // roll
        let translation_matrix = mat4x4_translate(position[0], position[1], position[2]);
        
        // Combine transformations: T * R * S
        let rotation_matrix = mat4x4_mul(mat4x4_mul(rotation_y, rotation_x), rotation_z);
        let transform_matrix = mat4x4_mul(rotation_matrix, scale_matrix);
        self.matrix = mat4x4_mul(translation_matrix, transform_matrix);
    }

    /// Extract position from the transformation matrix
    pub fn get_position(&self) -> [f32; 3] {
        mat4x4_extract_translation(&self.matrix)
    }

    /// Extract scale from the transformation matrix
    pub fn get_scale(&self) -> [f32; 3] {
        mat4x4_extract_scale(&self.matrix)
    }

    /// Extract euler angles from the transformation matrix
    pub fn get_rotation(&self) -> [f32; 3] {
        mat4x4_extract_euler_angles(&self.matrix)
    }
}
