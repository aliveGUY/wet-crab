// Transform component for 3D objects
pub struct Transform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4], // quaternion
    pub scale: [f32; 3],
    transform_matrix: Option<[f32; 16]>, // cached matrix
    dirty: bool,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0], // identity quaternion
            scale: [1.0, 1.0, 1.0],
            transform_matrix: None,
            dirty: true,
        }
    }

    pub fn from_components(translation: [f32; 3], rotation: [f32; 4], scale: [f32; 3]) -> Self {
        Self {
            translation,
            rotation,
            scale,
            transform_matrix: None,
            dirty: true,
        }
    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.translation[0] += x;
        self.translation[1] += y;
        self.translation[2] += z;
        self.dirty = true;
    }

    pub fn set_translation(&mut self, x: f32, y: f32, z: f32) {
        self.translation = [x, y, z];
        self.dirty = true;
    }

    pub fn set_rotation(&mut self, x: f32, y: f32, z: f32, w: f32) {
        self.rotation = [x, y, z, w];
        self.dirty = true;
    }

    pub fn set_scale(&mut self, x: f32, y: f32, z: f32) {
        self.scale = [x, y, z];
        self.dirty = true;
    }

    pub fn set_rotation_y(&mut self, angle_radians: f32) {
        // Create quaternion for Y-axis rotation
        let half_angle = angle_radians * 0.5;
        let sin_half = half_angle.sin();
        let cos_half = half_angle.cos();
        self.rotation = [0.0, sin_half, 0.0, cos_half]; // [x, y, z, w]
        self.dirty = true;
    }

    pub fn set_rotation_x(&mut self, angle_radians: f32) {
        // Create quaternion for X-axis rotation
        let half_angle = angle_radians * 0.5;
        let sin_half = half_angle.sin();
        let cos_half = half_angle.cos();
        self.rotation = [sin_half, 0.0, 0.0, cos_half]; // [x, y, z, w]
        self.dirty = true;
    }

    pub fn get_matrix(&mut self) -> [f32; 16] {
        if self.dirty || self.transform_matrix.is_none() {
            self.calculate_matrix();
        }
        self.transform_matrix.unwrap()
    }

    fn calculate_matrix(&mut self) {
        // We need to import math functions in the parent scope
        // For now, we'll implement the basic matrix calculation inline
        // Scale -> Rotate -> Translate (SRT order)
        
        // Start with scale matrix
        let mut matrix = [
            self.scale[0], 0.0, 0.0, 0.0,
            0.0, self.scale[1], 0.0, 0.0,
            0.0, 0.0, self.scale[2], 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];
        
        // Apply rotation (quaternion to matrix conversion)
        let [x, y, z, w] = self.rotation;
        let x2 = x * x;
        let y2 = y * y;
        let z2 = z * z;
        let w2 = w * w;
        
        let xy = 2.0 * x * y;
        let xz = 2.0 * x * z;
        let xw = 2.0 * x * w;
        let yz = 2.0 * y * z;
        let yw = 2.0 * y * w;
        let zw = 2.0 * z * w;
        
        let rotation_matrix = [
            w2 + x2 - y2 - z2, xy - zw,            xz + yw,            0.0,
            xy + zw,            w2 - x2 + y2 - z2, yz - xw,            0.0,
            xz - yw,            yz + xw,            w2 - x2 - y2 + z2, 0.0,
            0.0,                0.0,                0.0,                1.0,
        ];
        
        // Multiply scale * rotation
        let mut temp = [0.0; 16];
        for i in 0..4 {
            for j in 0..4 {
                temp[i * 4 + j] = 0.0;
                for k in 0..4 {
                    temp[i * 4 + j] += rotation_matrix[i * 4 + k] * matrix[k * 4 + j];
                }
            }
        }
        
        // Apply translation
        matrix = temp;
        matrix[3] = self.translation[0];
        matrix[7] = self.translation[1];
        matrix[11] = self.translation[2];
        
        self.transform_matrix = Some(matrix);
        self.dirty = false;
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}
