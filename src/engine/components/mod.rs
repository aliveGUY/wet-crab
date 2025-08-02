pub mod animated_object3d;
pub mod animation_state;
pub mod animator;
pub mod camera;
pub mod material;
pub mod mesh;
pub mod shared_components;
pub mod shapes;
pub mod skeleton;
pub mod static_object3d;
pub mod system;
pub mod transform;
pub mod metadata;
pub mod collider;

// Re-export commonly used types for convenience
pub use camera::Camera as CameraComponent;
pub use system::SystemTrait;
pub use transform::Transform;
pub use metadata::Metadata;
pub use shapes::Shape;
pub use collider::{Collider, ColliderLayer};
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
