use crate::index::math::{Mat4x4, build_view_matrix};
use crate::index::Transform;
use std::sync::RwLock;

#[derive(Debug)]
pub struct Camera {
    transform: RwLock<Transform>,
    position: RwLock<[f32; 3]>,
    pitch: RwLock<f32>,
    yaw: RwLock<f32>,
    roll: RwLock<f32>,
    transform_dirty: RwLock<bool>,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            transform: RwLock::new(Transform::new()),
            position: RwLock::new([0.0, 0.0, 0.0]),
            pitch: RwLock::new(0.0),
            yaw: RwLock::new(0.0),
            roll: RwLock::new(0.0),
            transform_dirty: RwLock::new(true),
        }
    }

    /// Get the view matrix - always up-to-date, read-only access
    pub fn get_view_matrix(&self) -> Mat4x4 {
        // Check if update needed
        if *self.transform_dirty.read().unwrap() {
            self.update_transform_matrix();
        }
        *self.transform.read().unwrap().get_matrix()
    }

    /// Set camera position directly
    pub fn set_position(&self, x: f32, y: f32, z: f32) {
        *self.position.write().unwrap() = [x, y, z];
        *self.transform_dirty.write().unwrap() = true;
    }

    /// Add rotation delta for mouse look
    pub fn add_rotation_delta(&self, pitch_delta: f32, yaw_delta: f32) {
        *self.yaw.write().unwrap() += yaw_delta;
        *self.pitch.write().unwrap() += pitch_delta;
        
        // Clamp pitch to prevent gimbal lock
        let mut pitch = self.pitch.write().unwrap();
        *pitch = pitch.clamp(-1.5, 1.5);
        
        *self.transform_dirty.write().unwrap() = true;
    }

    /// Move camera relative to its current orientation
    pub fn move_relative(&self, forward: f32, right: f32, up: f32) {
        let (f, r, u) = self.basis_from_yaw();
        
        // Update position using RwLock
        {
            let mut position = self.position.write().unwrap();
            position[0] += forward * f[0] + right * r[0] + up * u[0];
            position[1] += forward * f[1] + right * r[1] + up * u[1];
            position[2] += forward * f[2] + right * r[2] + up * u[2];
        }
        
        *self.transform_dirty.write().unwrap() = true;
    }

    /// Movement helper methods
    pub fn move_forward(&self, step: f32) {
        self.move_relative(-step, 0.0, 0.0);
    }

    pub fn move_back(&self, step: f32) {
        self.move_relative(step, 0.0, 0.0);
    }

    pub fn move_right(&self, step: f32) {
        self.move_relative(0.0, step, 0.0);
    }

    pub fn move_left(&self, step: f32) {
        self.move_relative(0.0, -step, 0.0);
    }

    pub fn move_up(&self, step: f32) {
        self.move_relative(0.0, 0.0, step);
    }

    pub fn move_down(&self, step: f32) {
        self.move_relative(0.0, 0.0, -step);
    }

    pub fn move_forward_right(&self, step: f32) {
        let s = step * 0.70710677; // sqrt(2)/2 for diagonal movement
        self.move_relative(-s, s, 0.0);
    }

    pub fn move_forward_left(&self, step: f32) {
        let s = step * 0.70710677;
        self.move_relative(-s, -s, 0.0);
    }

    pub fn move_back_right(&self, step: f32) {
        let s = step * 0.70710677;
        self.move_relative(s, s, 0.0);
    }

    pub fn move_back_left(&self, step: f32) {
        let s = step * 0.70710677;
        self.move_relative(s, -s, 0.0);
    }

    /// Private helper methods
    fn update_transform_matrix(&self) {
        if *self.transform_dirty.read().unwrap() {
            // Build new view matrix using the stored position
            let position = *self.position.read().unwrap();
            let pitch = *self.pitch.read().unwrap();
            let yaw = *self.yaw.read().unwrap();
            let view_matrix = build_view_matrix(position, pitch, yaw);
            
            // Update transform with new matrix
            let mut transform = self.transform.write().unwrap();
            *transform = Transform::new();
            // Note: We're storing the view matrix directly in the transform
            // This is a bit of a hack, but maintains compatibility
            *transform.get_matrix_mut() = view_matrix;
            
            *self.transform_dirty.write().unwrap() = false;
        }
    }

    fn basis_from_yaw(&self) -> ([f32; 3], [f32; 3], [f32; 3]) {
        let yaw = *self.yaw.read().unwrap();
        let cy = yaw.cos();
        let sy = yaw.sin();

        let forward = [-sy, 0.0, cy];
        let right = [cy, 0.0, sy];
        let up = [0.0, 1.0, 0.0];

        (forward, right, up)
    }
}

impl Clone for Camera {
    fn clone(&self) -> Self {
        Self {
            transform: RwLock::new(self.transform.read().unwrap().clone()),
            position: RwLock::new(*self.position.read().unwrap()),
            pitch: RwLock::new(*self.pitch.read().unwrap()),
            yaw: RwLock::new(*self.yaw.read().unwrap()),
            roll: RwLock::new(*self.roll.read().unwrap()),
            transform_dirty: RwLock::new(*self.transform_dirty.read().unwrap()),
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
