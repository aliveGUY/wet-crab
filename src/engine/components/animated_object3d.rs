// Import shared components
use crate::index::engine::{
    components::shared_components::{Material, Mesh},
    managers::assets_manager::{Assets, get_animated_object_copy},
};
use serde::{Serialize, Deserialize, Deserializer};

// Import animation-specific components
mod skeleton_mod {
    include!("skeleton.rs");
}
mod animation_mod {
    include!("animation_state.rs");
}
mod animator_mod {
    include!("animator.rs");
}

pub use skeleton_mod::*;
pub use animation_mod::*;
pub use animator_mod::Animator;

#[derive(Serialize, Clone, Debug)]
pub struct AnimatedObject3D {
    pub asset_type: Assets, // Serializable asset identifier
    #[serde(skip)]
    pub mesh: Mesh,
    #[serde(skip)]
    pub material: Material, // Required, no Option
    #[serde(skip)]
    pub skeleton: Skeleton, // Required, no Option
    #[serde(skip)]
    pub animation_channels: Vec<AnimationChannel>, // Required
    #[serde(skip)]
    pub animator: Animator, // Required, now public for system access
}

// Helper struct for deserialization
#[derive(Deserialize)]
struct AnimatedObject3DHelper {
    asset_type: Assets,
}

// Custom deserialization that properly initializes from AssetManager
impl<'de> Deserialize<'de> for AnimatedObject3D {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the JSON structure to extract asset_type
        let helper = AnimatedObject3DHelper::deserialize(deserializer)?;
        
        // Use AssetManager to get the properly initialized object
        Ok(get_animated_object_copy(helper.asset_type))
    }
}

impl AnimatedObject3D {
    pub fn new(
        mesh: Mesh,
        material: Material,
        skeleton: Skeleton,
        animation_channels: Vec<AnimationChannel>,
        asset_type: Assets
    ) -> Self {
        Self {
            asset_type,
            mesh,
            material,
            skeleton,
            animation_channels,
            animator: Animator::new(),
        }
    }
}
