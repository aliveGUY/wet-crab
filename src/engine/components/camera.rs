use crate::index::engine::utils::math::{ Mat4x4, build_view_matrix, mat4x4_extract_translation };
use crate::index::engine::Component;
use crate::index::engine::components::SharedComponents::Transform;
use crate::{ ComponentUI, AttributeUI };
use slint::{ VecModel, ModelRc };
use std::sync::RwLock;
use std::rc::Rc;
use std::cell::RefCell;
use serde::{ Serialize, Deserialize };

#[derive(Debug)]
pub struct Camera {
    pitch: RwLock<f32>,
    yaw: RwLock<f32>,
    component_ui: Rc<RefCell<ComponentUI>>, // Single-threaded shared UI
}

// Custom serialization for Camera since RwLock can't be serialized
impl Serialize for Camera {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Camera", 2)?;
        state.serialize_field("pitch", &*self.pitch.read().unwrap())?;
        state.serialize_field("yaw", &*self.yaw.read().unwrap())?;
        state.end()
    }
}

impl Camera {
    pub fn new() -> Self {
        let attributes = vec![
            AttributeUI {
                name: "pitch (degrees)".into(),
                dt_type: "FLOAT".into(),
                value: "0.0".into(),
            },
            AttributeUI {
                name: "yaw (degrees)".into(),
                dt_type: "FLOAT".into(),
                value: "0.0".into(),
            }
        ];

        let component_ui = ComponentUI {
            name: "Camera".into(),
            attributes: ModelRc::new(VecModel::from(attributes)),
        };

        Self {
            pitch: RwLock::new(0.0),
            yaw: RwLock::new(0.0),
            component_ui: Rc::new(RefCell::new(component_ui)),
        }
    }

    /// Get the view matrix by combining entity Transform with camera orientation
    pub fn get_view_matrix(&self, entity_id: &str) -> Mat4x4 {
        // Get position from entity's Transform component
        let mut position = [0.0, 0.0, 0.0];
        let entity_id_string = entity_id.to_string();
        crate::query_by_id!(entity_id_string, (Transform), |transform| {
            let translation = mat4x4_extract_translation(transform.get_matrix());
            position = translation;
        });

        let pitch = *self.pitch.read().unwrap();
        let yaw = *self.yaw.read().unwrap();
        
        build_view_matrix(position, pitch, yaw)
    }

    /// Add rotation delta for mouse look
    pub fn add_rotation_delta(&self, pitch_delta: f32, yaw_delta: f32) {
        *self.yaw.write().unwrap() += yaw_delta;
        *self.pitch.write().unwrap() += pitch_delta;

        // Clamp pitch to prevent gimbal lock
        let mut pitch = self.pitch.write().unwrap();
        *pitch = pitch.clamp(-1.5, 1.5);
    }

    /// Get camera basis vectors for movement calculations
    pub fn get_basis_vectors(&self) -> ([f32; 3], [f32; 3], [f32; 3]) {
        let yaw = *self.yaw.read().unwrap();
        let cy = yaw.cos();
        let sy = yaw.sin();

        let forward = [-sy, 0.0, cy];
        let right = [cy, 0.0, sy];
        let up = [0.0, 1.0, 0.0];

        (forward, right, up)
    }
}

impl Clone for Camera {
    fn clone(&self) -> Self {
        Self {
            pitch: RwLock::new(*self.pitch.read().unwrap()),
            yaw: RwLock::new(*self.yaw.read().unwrap()),
            component_ui: Rc::new(RefCell::new(self.component_ui.borrow().clone())),
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Camera {
    fn apply_ui(&mut self, component_ui: &ComponentUI) {
        // Apply UI changes back to the camera
        use slint::Model;
        
        for i in 0..component_ui.attributes.row_count() {
            if let Some(attribute) = component_ui.attributes.row_data(i) {
                match attribute.name.as_str() {
                    "pitch (degrees)" => {
                        if let Ok(degrees) = attribute.value.parse::<f32>() {
                            *self.pitch.write().unwrap() = degrees.to_radians();
                        }
                    },
                    "yaw (degrees)" => {
                        if let Ok(degrees) = attribute.value.parse::<f32>() {
                            *self.yaw.write().unwrap() = degrees.to_radians();
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    fn update_component_ui(&mut self, entity_id: &str) {
        // Update SharedStrings in the Rc<RefCell<ComponentUI>> with current component values
        let mut ui = self.component_ui.borrow_mut();
        
        // Read actual values from the camera
        let pitch = *self.pitch.read().unwrap();
        let yaw = *self.yaw.read().unwrap();
        
        // Convert radians to degrees for better UI display
        let pitch_degrees = pitch.to_degrees();
        let yaw_degrees = yaw.to_degrees();

        // Update existing SharedStrings in-place
        use slint::Model;
        for i in 0..ui.attributes.row_count() {
            if let Some(mut attr) = ui.attributes.row_data(i) {
                match attr.name.as_str() {
                    "pitch (degrees)" => attr.value = format!("{:.1}", pitch_degrees).into(),
                    "yaw (degrees)" => attr.value = format!("{:.1}", yaw_degrees).into(),
                    _ => {}
                }
                ui.attributes.set_row_data(i, attr);
            }
        }
        
        println!("🔄 Camera SharedStrings updated for entity {}", entity_id);
    }

    fn get_component_ui(&self) -> Rc<RefCell<ComponentUI>> {
        self.component_ui.clone()
    }
}

// Custom deserialization for Camera since RwLock can't be deserialized
impl<'de> Deserialize<'de> for Camera {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Deserializer, MapAccess, Visitor};
        use std::fmt;

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Pitch, Yaw }

        struct CameraVisitor;

        impl<'de> Visitor<'de> for CameraVisitor {
            type Value = Camera;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Camera")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Camera, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut pitch = None;
                let mut yaw = None;
                
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Pitch => {
                            if pitch.is_some() {
                                return Err(de::Error::duplicate_field("pitch"));
                            }
                            pitch = Some(map.next_value()?);
                        }
                        Field::Yaw => {
                            if yaw.is_some() {
                                return Err(de::Error::duplicate_field("yaw"));
                            }
                            yaw = Some(map.next_value()?);
                        }
                    }
                }
                
                let pitch = pitch.ok_or_else(|| de::Error::missing_field("pitch"))?;
                let yaw = yaw.ok_or_else(|| de::Error::missing_field("yaw"))?;
                
                // Create camera and set values
                let camera = Camera::new();
                *camera.pitch.write().unwrap() = pitch;
                *camera.yaw.write().unwrap() = yaw;
                
                Ok(camera)
            }
        }

        const FIELDS: &'static [&'static str] = &["pitch", "yaw"];
        deserializer.deserialize_struct("Camera", FIELDS, CameraVisitor)
    }
}
