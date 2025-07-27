use crate::index::engine::Component;
use crate::{ ComponentUI, AttributeUI };
use slint::{ VecModel, ModelRc };
use std::rc::Rc;
use std::cell::RefCell;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Clone, Debug)]
pub struct Metadata {
    title: String,
    role: Option<String>, // Entity role for global variable binding
    #[serde(skip)]
    component_ui: Rc<RefCell<ComponentUI>>, // Single-threaded shared UI
}

// Custom deserialization to properly initialize component UI
impl<'de> Deserialize<'de> for Metadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Deserializer, MapAccess, Visitor};
        use std::fmt;

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Title, Role }

        struct MetadataVisitor;

        impl<'de> Visitor<'de> for MetadataVisitor {
            type Value = Metadata;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Metadata")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Metadata, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut title: Option<String> = None;
                let mut role: Option<Option<String>> = None;
                
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Title => {
                            if title.is_some() {
                                return Err(de::Error::duplicate_field("title"));
                            }
                            title = Some(map.next_value()?);
                        }
                        Field::Role => {
                            if role.is_some() {
                                return Err(de::Error::duplicate_field("role"));
                            }
                            role = Some(map.next_value()?);
                        }
                    }
                }
                
                let title = title.ok_or_else(|| de::Error::missing_field("title"))?;
                let role = role.unwrap_or(None);
                
                // Create new component with proper UI initialization
                Ok(Metadata::new_with_role(&title, role.as_deref()))
            }
        }

        const FIELDS: &'static [&'static str] = &["title", "role"];
        deserializer.deserialize_struct("Metadata", FIELDS, MetadataVisitor)
    }
}

impl Metadata {
    pub fn new(title: &str) -> Self {
        Self::new_with_role(title, None)
    }

    pub fn new_with_role(title: &str, role: Option<&str>) -> Self {
        let mut attributes = vec![AttributeUI {
            name: "title".into(),
            dt_type: "STRING".into(),
            value: title.into(),
        }];

        // Add role attribute if present
        if let Some(role_str) = role {
            attributes.push(AttributeUI {
                name: "role".into(),
                dt_type: "STRING".into(),
                value: role_str.into(),
            });
        }

        let component_ui = ComponentUI {
            name: "Metadata".into(),
            attributes: ModelRc::new(VecModel::from(attributes)),
        };

        Self {
            title: title.to_string(),
            role: role.map(|r| r.to_string()),
            component_ui: Rc::new(RefCell::new(component_ui)),
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn role(&self) -> Option<&str> {
        self.role.as_deref()
    }
}

impl Component for Metadata {
    fn apply_ui(&mut self, component_ui: &ComponentUI) {
        use slint::Model;
        
        for i in 0..component_ui.attributes.row_count() {
            if let Some(attribute) = component_ui.attributes.row_data(i) {
                if attribute.name.as_str() == "title" {
                    self.title = attribute.value.to_string();
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
                    "title" => attr.value = self.title.clone().into(),
                    "role" => attr.value = self.role.as_deref().unwrap_or("").into(),
                    _ => {}
                }
                ui.attributes.set_row_data(i, attr);
            }
        }
        
        println!("🔄 Metadata SharedStrings updated for entity {}", entity_id);
    }

    fn get_component_ui(&self) -> Rc<RefCell<ComponentUI>> {
        self.component_ui.clone()
    }
}
