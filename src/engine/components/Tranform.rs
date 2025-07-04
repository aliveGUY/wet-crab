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

    pub fn get_matrix(&mut self) -> [f32; 16] {
        if self.dirty || self.transform_matrix.is_none() {
            self.calculate_matrix();
        }
        self.transform_matrix.unwrap()
    }

    fn calculate_matrix(&mut self) {
        // This will need to use the math functions, but we'll handle the import later
        // For now, create identity matrix
        self.transform_matrix = Some([
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ]);
        self.dirty = false;
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}
