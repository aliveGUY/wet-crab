// Import shared components
use crate::index::engine::{
    components::SharedComponents::{Material, Mesh},
    managers::assets_manager::Assets,
};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StaticObject3D {
    pub asset_type: Assets, // Serializable asset identifier
    #[serde(skip)]
    pub mesh: Mesh,
    #[serde(skip)]
    pub material: Material, // Required, no Option
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
