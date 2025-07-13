// Import shared components
use crate::index::shared_components::{Mesh, Material};

// Import animation-specific components
mod skeleton_mod {
    include!("Skeleton.rs");
}
mod animation_mod {
    include!("AnimationState.rs");
}
mod animator_mod {
    include!("Animator.rs");
}

pub use skeleton_mod::*;
pub use animation_mod::*;
pub use animator_mod::Animator;

#[derive(Clone)]
pub struct AnimatedObject3D {
    pub mesh: Mesh,
    pub material: Material,  // Required, no Option
    pub skeleton: Skeleton,  // Required, no Option
    pub animation_channels: Vec<AnimationChannel>,  // Required
    pub animator: Animator,  // Required, now public for system access
}

impl AnimatedObject3D {
    pub fn new(
        mesh: Mesh, 
        material: Material, 
        skeleton: Skeleton, 
        animation_channels: Vec<AnimationChannel>
    ) -> Self {
        Self {
            mesh,
            material,
            skeleton,
            animation_channels,
            animator: Animator::new(),
        }
    }
}
