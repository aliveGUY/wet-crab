use serde::{Serialize, Deserialize};
use crate::index::engine::components::ComponentType;

/// Individual component in the scene format
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializedComponent {
    #[serde(rename = "type")]
    pub component_type: ComponentType,
    pub data: serde_json::Value,
}

/// Scene file format
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SceneFormat {
    pub scene_name: String,
    pub scene: Vec<Vec<SerializedComponent>>, // Array of entities, each containing array of components
}

impl Default for SceneFormat {
    fn default() -> Self {
        Self {
            scene_name: "no_name".to_string(),
            scene: Vec::new(),
        }
    }
}

impl SceneFormat {
    pub fn new(scene_name: &str) -> Self {
        Self {
            scene_name: scene_name.to_string(),
            scene: Vec::new(),
        }
    }

    pub fn add_entity(&mut self, components: Vec<SerializedComponent>) {
        self.scene.push(components);
    }

    pub fn entity_count(&self) -> usize {
        self.scene.len()
    }
}
