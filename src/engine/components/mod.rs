pub mod animated_object3d;
pub mod animation_state;
pub mod animator;
pub mod camera;
pub mod collider;
pub mod component_types;
pub mod material;
pub mod mesh;
pub mod metadata;
pub mod shared_components;
pub mod shapes;
pub mod skeleton;
pub mod static_object3d;
pub mod system;
pub mod transform;

// Re-export commonly used types for convenience
pub use camera::Camera as CameraComponent;
pub use collider::{Collider, ColliderLayer};
pub use component_types::ComponentType;
pub use metadata::Metadata;
pub use shapes::Shape;
pub use system::SystemTrait;
pub use transform::Transform;
#[allow(dead_code)]
pub type Material = self::material::Material;
#[allow(dead_code)]
pub type Mesh = self::mesh::Mesh;

// Re-export the main component types using aliases to avoid conflicts
pub use animated_object3d::AnimatedObject3D as AnimatedObject3DComponent;
pub use static_object3d::StaticObject3D as StaticObject3DComponent;

// Re-export modules for backward compatibility
pub use animated_object3d as AnimatedObject3D;
pub use shared_components as SharedComponents;
