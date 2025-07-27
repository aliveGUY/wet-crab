// Import shared components
use crate::index::engine::{
    components::shared_components::{ Material, Mesh },
    Component,
    managers::assets_manager::Assets,
};
use crate::{ ComponentUI, AttributeUI };
use slint::{ VecModel, ModelRc };
use std::rc::Rc;
use std::cell::RefCell;
use serde::{Serialize, Deserialize};

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
    #[serde(skip)]
    component_ui: Rc<RefCell<ComponentUI>>, // Single-threaded shared UI
}

// Custom deserialization to properly initialize component UI
impl<'de> Deserialize<'de> for AnimatedObject3D {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Deserializer, MapAccess, Visitor};
        use std::fmt;

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field { AssetType }

        struct AnimatedObject3DVisitor;

        impl<'de> Visitor<'de> for AnimatedObject3DVisitor {
            type Value = AnimatedObject3D;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct AnimatedObject3D")
            }

            fn visit_map<V>(self, mut map: V) -> Result<AnimatedObject3D, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut asset_type = None;
                
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::AssetType => {
                            if asset_type.is_some() {
                                return Err(de::Error::duplicate_field("asset_type"));
                            }
                            asset_type = Some(map.next_value()?);
                        }
                    }
                }
                
                let asset_type = asset_type.ok_or_else(|| de::Error::missing_field("asset_type"))?;
                
                // Restore OpenGL resources from asset manager and create properly initialized component
                use crate::index::engine::managers::assets_manager::get_animated_object_copy;
                let restored_obj = get_animated_object_copy(asset_type);
                
                // Create new component with proper UI initialization
                Ok(AnimatedObject3D::new(
                    restored_obj.mesh,
                    restored_obj.material,
                    restored_obj.skeleton,
                    restored_obj.animation_channels,
                    asset_type
                ))
            }
        }

        const FIELDS: &'static [&'static str] = &["asset_type"];
        deserializer.deserialize_struct("AnimatedObject3D", FIELDS, AnimatedObject3DVisitor)
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
        let attributes = vec![
            AttributeUI {
                name: "Asset Type".into(),
                dt_type: "STRING".into(),
                value: format!("{:?}", asset_type).into(),
            },
            AttributeUI {
                name: "Vertex Count".into(),
                dt_type: "INT".into(),
                value: mesh.vertex_count.to_string().into(),
            },
            AttributeUI {
                name: "Index Count".into(),
                dt_type: "INT".into(),
                value: mesh.index_count.to_string().into(),
            },
            AttributeUI {
                name: "Has Texture".into(),
                dt_type: "BOOL".into(),
                value: material.base_color_texture.is_some().to_string().into(),
            },
            AttributeUI {
                name: "Joint Count".into(),
                dt_type: "INT".into(),
                value: skeleton.nodes.len().to_string().into(),
            },
            AttributeUI {
                name: "Animation Channels".into(),
                dt_type: "INT".into(),
                value: animation_channels.len().to_string().into(),
            },
            AttributeUI {
                name: "Animation Speed".into(),
                dt_type: "FLOAT".into(),
                value: "30.0".into(),
            },
        ];
        
        let component_ui = ComponentUI {
            name: "Animated Object 3D".into(),
            attributes: ModelRc::new(VecModel::from(attributes)),
        };

        Self {
            asset_type,
            mesh,
            material,
            skeleton,
            animation_channels,
            animator: Animator::new(),
            component_ui: Rc::new(RefCell::new(component_ui)),
        }
    }

}

impl Component for AnimatedObject3D {
    fn apply_ui(&mut self, component_ui: &ComponentUI) {
        // Apply UI changes back to the component
        use slint::Model;
        
        for i in 0..component_ui.attributes.row_count() {
            if let Some(attribute) = component_ui.attributes.row_data(i) {
                match attribute.name.as_str() {
                    "Animation Speed" => {
                        if let Ok(speed) = attribute.value.parse::<f32>() {
                            self.animator.set_animation_speed(speed);
                        }
                    }
                    // Other attributes are read-only for now
                    _ => {}
                }
            }
        }
    }

    fn update_component_ui(&mut self, entity_id: &str) {
        // Update SharedStrings in the Rc<RefCell<ComponentUI>> with current component values
        let mut ui = self.component_ui.borrow_mut();
        
        // Update existing SharedStrings in-place
        use slint::Model;
        for i in 0..ui.attributes.row_count() {
            if let Some(mut attr) = ui.attributes.row_data(i) {
                match attr.name.as_str() {
                    "Asset Type" => attr.value = format!("{:?}", self.asset_type).into(),
                    "Vertex Count" => attr.value = self.mesh.vertex_count.to_string().into(),
                    "Index Count" => attr.value = self.mesh.index_count.to_string().into(),
                    "Has Texture" => attr.value = self.material.base_color_texture.is_some().to_string().into(),
                    "Joint Count" => attr.value = self.skeleton.nodes.len().to_string().into(),
                    "Animation Channels" => attr.value = self.animation_channels.len().to_string().into(),
                    "Animation Speed" => attr.value = self.animator.get_animation_speed().to_string().into(),
                    _ => {}
                }
                ui.attributes.set_row_data(i, attr);
            }
        }
        
        println!("🔄 AnimatedObject3D SharedStrings updated for entity {}", entity_id);
    }

    fn get_component_ui(&self) -> Rc<RefCell<ComponentUI>> {
        self.component_ui.clone()
    }
}
