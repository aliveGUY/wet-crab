// Mesh component for 3D objects
pub struct Mesh {
    pub vao: glow::VertexArray,
    pub index_count: usize,
    pub vertex_count: usize,
}

impl Mesh {
    pub fn new() -> Self {
        // Create a default/empty mesh - this will be replaced with actual data
        Self {
            vao: unsafe { std::mem::zeroed() }, // Will be properly initialized when loading model
            index_count: 0,
            vertex_count: 0,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.index_count > 0 && self.vertex_count > 0
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}
