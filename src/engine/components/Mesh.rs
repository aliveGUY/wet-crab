// Mesh component for 3D objects
pub struct Mesh {
    pub vao: Option<u32>, // Vertex Array Object ID
    pub vertex_buffer: Option<u32>,
    pub index_buffer: Option<u32>,
    pub num_indices: usize,
    pub num_vertices: usize,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            vao: None,
            vertex_buffer: None,
            index_buffer: None,
            num_indices: 0,
            num_vertices: 0,
        }
    }

    pub fn with_buffers(vao: u32, vertex_buffer: u32, index_buffer: u32, num_indices: usize, num_vertices: usize) -> Self {
        Self {
            vao: Some(vao),
            vertex_buffer: Some(vertex_buffer),
            index_buffer: Some(index_buffer),
            num_indices,
            num_vertices,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.vao.is_some() && self.num_indices > 0
    }

    pub fn bind_vao(&self) -> Option<u32> {
        self.vao
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}
