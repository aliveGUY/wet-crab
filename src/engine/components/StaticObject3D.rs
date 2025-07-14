// Import shared components
use crate::index::engine::components::{MeshComponent, MaterialComponent};

#[derive(Clone)]
pub struct StaticObject3D {
    pub mesh: MeshComponent,
    pub material: MaterialComponent,  // Required, no Option
}

impl StaticObject3D {
    pub fn new(mesh: MeshComponent, material: MaterialComponent) -> Self {
        Self {
            mesh,
            material,
        }
    }
}
