pub mod AnimatedObject3D;
pub mod AnimationState;
pub mod Animator;
pub mod Camera;
pub mod Material;
pub mod Mesh;
pub mod SharedComponents;
pub mod Skeleton;
pub mod StaticObject3D;
pub mod System;
pub mod Transform;

// Re-export shared components (which includes Transform, Mesh, Material)
pub use SharedComponents::*;

// Re-export the main component types using aliases to avoid conflicts
pub use AnimatedObject3D::AnimatedObject3D as AnimatedObject3DComponent;
pub use Camera::Camera as CameraComponent;
pub use StaticObject3D::StaticObject3D as StaticObject3DComponent;
pub use System::System as SystemTrait;
