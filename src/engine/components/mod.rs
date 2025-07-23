pub mod AnimatedObject3D;
pub mod AnimationState;
pub mod Animator;
pub mod Camera;
pub mod material;
pub mod mesh;
pub mod SharedComponents;
pub mod Skeleton;
pub mod StaticObject3D;
pub mod System;
pub mod transform;
pub mod metadata;

// Re-export all modules for direct access (only what's actually used)

// Create type aliases for direct access to structs (this is the key fix!)
pub type Transform = self::transform::Transform;
pub type Metadata = self::metadata::Metadata;
pub type Material = self::material::Material;
pub type Mesh = self::mesh::Mesh;

// Re-export the main component types using aliases to avoid conflicts
pub use AnimatedObject3D::AnimatedObject3D as AnimatedObject3DComponent;
pub use Camera::Camera as CameraComponent;
pub use StaticObject3D::StaticObject3D as StaticObject3DComponent;
pub use System::System as SystemTrait;
