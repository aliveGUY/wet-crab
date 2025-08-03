use crate::index::engine::utils::math::{Mat4x4, build_view_matrix, mat4x4_extract_translation};
use crate::index::engine::components::SharedComponents::Transform;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Camera {
    pub pitch: f32,
    pub yaw: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            pitch: 0.0,
            yaw: 0.0,
        }
    }

    /// Get the view matrix by combining entity Transform with camera orientation
    pub fn get_view_matrix(&self, entity_id: &str) -> Mat4x4 {
        // Get position from entity's Transform component
        let mut position = [0.0, 0.0, 0.0];
        let entity_id_string = entity_id.to_string();
        crate::query_by_id!(entity_id_string, (Transform), |transform| {
            let translation = mat4x4_extract_translation(transform.get_matrix());
            position = translation;
        });

        build_view_matrix(position, self.pitch, self.yaw)
    }

    /// Add rotation delta for mouse look
    pub fn add_rotation_delta(&mut self, pitch_delta: f32, yaw_delta: f32) {
        self.yaw += yaw_delta;
        self.pitch += pitch_delta;

        // Clamp pitch to prevent gimbal lock
        self.pitch = self.pitch.clamp(-1.5, 1.5);
    }

    /// Get camera basis vectors for movement calculations
    pub fn get_basis_vectors(&self) -> ([f32; 3], [f32; 3], [f32; 3]) {
        let cy = self.yaw.cos();
        let sy = self.yaw.sin();

        let forward = [-sy, 0.0, cy];
        let right = [cy, 0.0, sy];
        let up = [0.0, 1.0, 0.0];

        (forward, right, up)
    }

    /// Set pitch in radians
    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch.clamp(-1.5, 1.5);
    }

    /// Set yaw in radians
    pub fn set_yaw(&mut self, yaw: f32) {
        self.yaw = yaw;
    }

    /// Get pitch in radians
    pub fn get_pitch(&self) -> f32 {
        self.pitch
    }

    /// Get yaw in radians
    pub fn get_yaw(&self) -> f32 {
        self.yaw
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
