use crate::index::engine::utils::math::{
    mat4x4_from_quat, mat4x4_mul, mat4x4_translate, mat4x4_scale, Mat4x4,
    mat4x4_extract_translation, mat4x4_extract_scale, mat4x4_extract_euler_angles,
    mat4x4_rot_x, mat4x4_rot_y, mat4x4_rot_z, mat4x4_identity
};
use serde::{Serialize, Deserialize};

// Transform component for 3D objects - component-based approach
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transform {
    // Position components
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    
    // Scale components  
    pub scale_x: f32,
    pub scale_y: f32,
    pub scale_z: f32,
    
    // Rotation components (Euler angles in radians)
    pub rotation_x: f32, // pitch
    pub rotation_y: f32, // yaw
    pub rotation_z: f32, // roll
    
    // Cached matrix (not serialized, computed on demand)
    #[serde(skip)]
    cached_matrix: Option<Mat4x4>,
    #[serde(skip)]
    matrix_dirty: bool,
}

impl Transform {
    /// Create a new Transform with default values (identity transform)
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            position_x: x,
            position_y: y,
            position_z: z,
            scale_x: 1.0,
            scale_y: 1.0,
            scale_z: 1.0,
            rotation_x: 0.0,
            rotation_y: 0.0,
            rotation_z: 0.0,
            cached_matrix: None,
            matrix_dirty: true,
        }
    }

    /// Create a new identity Transform (position at origin, unit scale, no rotation)
    pub fn identity() -> Self {
        Self {
            position_x: 0.0,
            position_y: 0.0,
            position_z: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            scale_z: 1.0,
            rotation_x: 0.0,
            rotation_y: 0.0,
            rotation_z: 0.0,
            cached_matrix: None,
            matrix_dirty: true,
        }
    }

    /// Get the transformation matrix (cached for performance)
    /// Order: Scale -> Rotate -> Translate (SRT)
    pub fn get_matrix(&mut self) -> &Mat4x4 {
        if self.matrix_dirty || self.cached_matrix.is_none() {
            // Create individual transformation matrices
            let scale_matrix = mat4x4_scale(self.scale_x, self.scale_y, self.scale_z);
            let rotation_x = mat4x4_rot_x(self.rotation_x); // pitch
            let rotation_y = mat4x4_rot_y(self.rotation_y); // yaw  
            let rotation_z = mat4x4_rot_z(self.rotation_z); // roll
            let translation_matrix = mat4x4_translate(self.position_x, self.position_y, self.position_z);
            
            // Combine: T * R * S (right to left multiplication)
            let rotation_matrix = mat4x4_mul(mat4x4_mul(rotation_y, rotation_x), rotation_z);
            let transform_matrix = mat4x4_mul(rotation_matrix, scale_matrix);
            let final_matrix = mat4x4_mul(translation_matrix, transform_matrix);
            
            self.cached_matrix = Some(final_matrix);
            self.matrix_dirty = false;
        }
        
        self.cached_matrix.as_ref().unwrap()
    }

    /// Set position components
    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.position_x = x;
        self.position_y = y;
        self.position_z = z;
        self.matrix_dirty = true;
    }

    /// Set scale components
    pub fn set_scale(&mut self, x: f32, y: f32, z: f32) {
        self.scale_x = x;
        self.scale_y = y;
        self.scale_z = z;
        self.matrix_dirty = true;
    }

    /// Set rotation components (Euler angles in radians)
    pub fn set_rotation(&mut self, x: f32, y: f32, z: f32) {
        self.rotation_x = x;
        self.rotation_y = y;
        self.rotation_z = z;
        self.matrix_dirty = true;
    }

    /// Get position as array
    pub fn get_position(&self) -> [f32; 3] {
        [self.position_x, self.position_y, self.position_z]
    }

    /// Get scale as array
    pub fn get_scale(&self) -> [f32; 3] {
        [self.scale_x, self.scale_y, self.scale_z]
    }

    /// Get rotation as array (Euler angles in radians)
    pub fn get_rotation(&self) -> [f32; 3] {
        [self.rotation_x, self.rotation_y, self.rotation_z]
    }

    /// Apply translation to the current position
    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.position_x += x;
        self.position_y += y;
        self.position_z += z;
        self.matrix_dirty = true;
    }

    /// Apply rotation to the current rotation (adds to existing rotation)
    pub fn rotate(&mut self, x: f32, y: f32, z: f32, _w: f32) {
        // Note: This now takes Euler angles instead of quaternion
        // The w parameter is ignored for backward compatibility
        self.rotation_x += x;
        self.rotation_y += y;
        self.rotation_z += z;
        self.matrix_dirty = true;
    }

    /// Set the transform from position, rotation (euler angles in radians), and scale arrays
    pub fn set_from_components(&mut self, position: [f32; 3], rotation: [f32; 3], scale: [f32; 3]) {
        self.position_x = position[0];
        self.position_y = position[1];
        self.position_z = position[2];
        self.rotation_x = rotation[0];
        self.rotation_y = rotation[1];
        self.rotation_z = rotation[2];
        self.scale_x = scale[0];
        self.scale_y = scale[1];
        self.scale_z = scale[2];
        self.matrix_dirty = true;
    }

    /// Generate transformation matrix without caching (for read-only access)
    pub fn compute_matrix(&self) -> Mat4x4 {
        // Create individual transformation matrices
        let scale_matrix = mat4x4_scale(self.scale_x, self.scale_y, self.scale_z);
        let rotation_x = mat4x4_rot_x(self.rotation_x); // pitch
        let rotation_y = mat4x4_rot_y(self.rotation_y); // yaw  
        let rotation_z = mat4x4_rot_z(self.rotation_z); // roll
        let translation_matrix = mat4x4_translate(self.position_x, self.position_y, self.position_z);
        
        // Combine: T * R * S (right to left multiplication)
        let rotation_matrix = mat4x4_mul(mat4x4_mul(rotation_y, rotation_x), rotation_z);
        let transform_matrix = mat4x4_mul(rotation_matrix, scale_matrix);
        mat4x4_mul(translation_matrix, transform_matrix)
    }

    /// Create Transform from an existing matrix (for migration/compatibility)
    pub fn from_matrix(matrix: &Mat4x4) -> Self {
        let position = mat4x4_extract_translation(matrix);
        let scale = mat4x4_extract_scale(matrix);
        let rotation = mat4x4_extract_euler_angles(matrix);
        
        Self {
            position_x: position[0],
            position_y: position[1],
            position_z: position[2],
            scale_x: scale[0],
            scale_y: scale[1],
            scale_z: scale[2],
            rotation_x: rotation[0],
            rotation_y: rotation[1],
            rotation_z: rotation[2],
            cached_matrix: None,
            matrix_dirty: true,
        }
    }

    /// Get mutable reference to the transformation matrix (deprecated - use get_matrix())
    /// This method is kept for backward compatibility but now generates the matrix on-demand
    #[deprecated(note = "Use get_matrix() instead. Matrix is now generated from components.")]
    pub fn get_matrix_mut(&mut self) -> Mat4x4 {
        self.compute_matrix()
    }
}
