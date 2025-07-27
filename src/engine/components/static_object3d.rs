// Import shared components
use crate::index::engine::{
    components::SharedComponents::{ Material, Mesh },
    Component,
    managers::assets_manager::Assets,
};
use crate::{ ComponentUI, AttributeUI };
use slint::{ VecModel, ModelRc };
use std::rc::Rc;
use std::cell::RefCell;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Clone, Debug)]
pub struct StaticObject3D {
    pub asset_type: Assets, // Serializable asset identifier
    #[serde(skip)]
    pub mesh: Mesh,
    #[serde(skip)]
    pub material: Material, // Required, no Option
    #[serde(skip)]
    component_ui: Rc<RefCell<ComponentUI>>, // Single-threaded shared UI
}

// Custom deserialization to properly initialize component UI
impl<'de> Deserialize<'de> for StaticObject3D {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Deserializer, MapAccess, Visitor};
        use std::fmt;

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field { AssetType }

        struct StaticObject3DVisitor;

        impl<'de> Visitor<'de> for StaticObject3DVisitor {
            type Value = StaticObject3D;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct StaticObject3D")
            }

            fn visit_map<V>(self, mut map: V) -> Result<StaticObject3D, V::Error>
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
                use crate::index::engine::managers::assets_manager::get_static_object_copy;
                let restored_obj = get_static_object_copy(asset_type);
                
                // Create new component with proper UI initialization
                Ok(StaticObject3D::new(restored_obj.mesh, restored_obj.material, asset_type))
            }
        }

        const FIELDS: &'static [&'static str] = &["asset_type"];
        deserializer.deserialize_struct("StaticObject3D", FIELDS, StaticObject3DVisitor)
    }
}

impl StaticObject3D {
    pub fn new(mesh: Mesh, material: Material, asset_type: Assets) -> Self {
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
        ];
        
        let component_ui = ComponentUI {
            name: "Static Object 3D".into(),
            attributes: ModelRc::new(VecModel::from(attributes)),
        };

        Self {
            asset_type,
            mesh,
            material,
            component_ui: Rc::new(RefCell::new(component_ui)),
        }
    }

}

impl Component for StaticObject3D {
    fn apply_ui(&mut self, component_ui: &ComponentUI) {
        // Apply UI changes back to the component
        use slint::Model;
        
        for i in 0..component_ui.attributes.row_count() {
            if let Some(_attribute) = component_ui.attributes.row_data(i) {
                // TODO: Implement UI application logic when attributes are added
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
                    _ => {}
                }
                ui.attributes.set_row_data(i, attr);
            }
        }
        
        println!("🔄 StaticObject3D SharedStrings updated for entity {}", entity_id);
    }

    fn get_component_ui(&self) -> Rc<RefCell<ComponentUI>> {
        self.component_ui.clone()
    }
}
