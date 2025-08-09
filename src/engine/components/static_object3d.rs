// Import shared components
use crate::index::engine::{
    components::SharedComponents::{Material, Mesh},
    managers::assets_manager::{Assets, get_static_object_copy},
};
use serde::{Serialize, Deserialize, Deserializer};

#[derive(Serialize, Clone, Debug)]
pub struct StaticObject3D {
    pub asset_type: Assets, // Serializable asset identifier
    #[serde(skip)]
    pub mesh: Mesh,
    #[serde(skip)]
    pub material: Material, // Required, no Option
}

// Helper struct for deserialization
#[derive(Deserialize)]
struct StaticObject3DHelper {
    asset_type: Assets,
}

// Custom deserialization that properly initializes from AssetManager
impl<'de> Deserialize<'de> for StaticObject3D {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the JSON structure to extract asset_type
        let helper = StaticObject3DHelper::deserialize(deserializer)?;
        
        // Use AssetManager to get the properly initialized object
        Ok(get_static_object_copy(helper.asset_type))
    }
}

impl StaticObject3D {
    pub fn new(mesh: Mesh, material: Material, asset_type: Assets) -> Self {
        Self {
            asset_type,
            mesh,
            material,
        }
    }
}
