// Import shared components
use crate::index::engine::components::SharedComponents::{Mesh, Material};

#[derive(Clone)]
pub struct StaticObject3D {
    pub mesh: Mesh,
    pub material: Material,  // Required, no Option
}

impl StaticObject3D {
    pub fn new(mesh: Mesh, material: Material) -> Self {
        Self {
            mesh,
            material,
        }
    }
}
