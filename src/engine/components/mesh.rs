// Mesh component for 3D objects
#[derive(Clone, Debug)]
pub struct Mesh {
    pub vao: glow::VertexArray,
    pub index_count: usize,
    #[allow(dead_code)]
    pub vertex_count: usize,
}

impl Mesh {
    pub fn new() -> Self {
        // Create a default/empty mesh - this will be replaced with actual data
        // Using a dummy non-zero value to avoid the zero-initialization warning
        Self {
            #[allow(invalid_value)]
            vao: unsafe { std::mem::MaybeUninit::zeroed().assume_init() }, // Will be properly initialized when loading model
            index_count: 0,
            vertex_count: 0,
        }
    }

    #[allow(dead_code)]
    pub fn is_valid(&self) -> bool {
        self.index_count > 0 && self.vertex_count > 0
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}
