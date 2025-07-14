// Shared components that can be used by both StaticObject3D and AnimatedObject3D

// Re-export the shared components
mod transform_mod {
    include!("Transform.rs");
}
mod mesh_mod {
    include!("Mesh.rs");
}
mod material_mod {
    include!("Material.rs");
}

pub use transform_mod::Transform;
pub use mesh_mod::Mesh;
pub use material_mod::Material;
