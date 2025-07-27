use crate::index::engine::utils::math::{ Mat4x4, build_view_matrix };
use crate::index::engine::components::SharedComponents::Transform;
use crate::index::engine::Component;
use crate::{ ComponentUI, AttributeUI };
use slint::{ VecModel, ModelRc };
use std::sync::RwLock;
use std::rc::Rc;
use std::cell::RefCell;
use serde::{ Serialize, Deserialize };

#[derive(Debug)]
pub struct Camera {
    transform: RwLock<Transform>,
    position: RwLock<[f32; 3]>,
    pitch: RwLock<f32>,
    yaw: RwLock<f32>,
    roll: RwLock<f32>,
    transform_dirty: RwLock<bool>,
    component_ui: Rc<RefCell<ComponentUI>>, // Single-threaded shared UI
}

// Custom serialization for Camera since RwLock can't be serialized
impl Serialize for Camera {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Camera", 3)?;
        state.serialize_field("position", &*self.position.read().unwrap())?;
        state.serialize_field("pitch", &*self.pitch.read().unwrap())?;
        state.serialize_field("yaw", &*self.yaw.read().unwrap())?;
        state.end()
    }
}

impl Camera {
    pub fn new() -> Self {
        let attributes = vec![
            AttributeUI {
                name: "x position".into(),
                dt_type: "FLOAT".into(),
                value: "0.0".into(),
            },
            AttributeUI {
                name: "y position".into(),
                dt_type: "FLOAT".into(),
                value: "0.0".into(),
            },
            AttributeUI {
                name: "z position".into(),
                dt_type: "FLOAT".into(),
                value: "0.0".into(),
            },
            AttributeUI {
                name: "pitch (degrees)".into(),
                dt_type: "FLOAT".into(),
                value: "0.0".into(),
            },
            AttributeUI {
                name: "yaw (degrees)".into(),
                dt_type: "FLOAT".into(),
                value: "0.0".into(),
            },
            AttributeUI {
                name: "roll (degrees)".into(),
                dt_type: "FLOAT".into(),
                value: "0.0".into(),
            }
        ];

        let component_ui = ComponentUI {
            name: "Camera".into(),
            attributes: ModelRc::new(VecModel::from(attributes)),
        };

        Self {
            transform: RwLock::new(Transform::new(0.0, 0.0, 0.0)),
            position: RwLock::new([0.0, 0.0, 0.0]),
            pitch: RwLock::new(0.0),
            yaw: RwLock::new(0.0),
            roll: RwLock::new(0.0),
            transform_dirty: RwLock::new(true),
            component_ui: Rc::new(RefCell::new(component_ui)),
        }
    }

    /// Get the view matrix - always up-to-date, read-only access
    pub fn get_view_matrix(&self) -> Mat4x4 {
        // Check if update needed
        if *self.transform_dirty.read().unwrap() {
            self.update_transform_matrix();
        }
        *self.transform.read().unwrap().get_matrix()
    }

    /// Set camera position directly
    pub fn set_position(&self, x: f32, y: f32, z: f32) {
        *self.position.write().unwrap() = [x, y, z];
        *self.transform_dirty.write().unwrap() = true;
    }

    /// Add rotation delta for mouse look
    pub fn add_rotation_delta(&self, pitch_delta: f32, yaw_delta: f32) {
        *self.yaw.write().unwrap() += yaw_delta;
        *self.pitch.write().unwrap() += pitch_delta;

        // Clamp pitch to prevent gimbal lock
        let mut pitch = self.pitch.write().unwrap();
        *pitch = pitch.clamp(-1.5, 1.5);

        *self.transform_dirty.write().unwrap() = true;
    }

    /// Move camera relative to its current orientation
    pub fn move_relative(&self, forward: f32, right: f32, up: f32) {
        let (f, r, u) = self.basis_from_yaw();

        // Update position using RwLock
        {
            let mut position = self.position.write().unwrap();
            position[0] += forward * f[0] + right * r[0] + up * u[0];
            position[1] += forward * f[1] + right * r[1] + up * u[1];
            position[2] += forward * f[2] + right * r[2] + up * u[2];
        }

        *self.transform_dirty.write().unwrap() = true;
    }

    /// Movement helper methods
    pub fn move_forward(&self, step: f32) {
        self.move_relative(-step, 0.0, 0.0);
    }

    pub fn move_back(&self, step: f32) {
        self.move_relative(step, 0.0, 0.0);
    }

    pub fn move_right(&self, step: f32) {
        self.move_relative(0.0, step, 0.0);
    }

    pub fn move_left(&self, step: f32) {
        self.move_relative(0.0, -step, 0.0);
    }

    pub fn move_up(&self, step: f32) {
        self.move_relative(0.0, 0.0, step);
    }

    pub fn move_down(&self, step: f32) {
        self.move_relative(0.0, 0.0, -step);
    }

    pub fn move_forward_right(&self, step: f32) {
        let s = step * 0.70710677; // sqrt(2)/2 for diagonal movement
        self.move_relative(-s, s, 0.0);
    }

    pub fn move_forward_left(&self, step: f32) {
        let s = step * 0.70710677;
        self.move_relative(-s, -s, 0.0);
    }

    pub fn move_back_right(&self, step: f32) {
        let s = step * 0.70710677;
        self.move_relative(s, s, 0.0);
    }

    pub fn move_back_left(&self, step: f32) {
        let s = step * 0.70710677;
        self.move_relative(s, -s, 0.0);
    }


    /// Private helper methods
    fn update_transform_matrix(&self) {
        if *self.transform_dirty.read().unwrap() {
            // Build new view matrix using the stored position
            let position = *self.position.read().unwrap();
            let pitch = *self.pitch.read().unwrap();
            let yaw = *self.yaw.read().unwrap();
            let view_matrix = build_view_matrix(position, pitch, yaw);

            // Update transform with new matrix
            let mut transform = self.transform.write().unwrap();
            *transform = Transform::new(0.0, 0.0, 0.0);
            // Note: We're storing the view matrix directly in the transform
            // This is a bit of a hack, but maintains compatibility
            *transform.get_matrix_mut() = view_matrix;

            *self.transform_dirty.write().unwrap() = false;
        }
    }

    fn basis_from_yaw(&self) -> ([f32; 3], [f32; 3], [f32; 3]) {
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
            transform: RwLock::new(self.transform.read().unwrap().clone()),
            position: RwLock::new(*self.position.read().unwrap()),
            pitch: RwLock::new(*self.pitch.read().unwrap()),
            yaw: RwLock::new(*self.yaw.read().unwrap()),
            roll: RwLock::new(*self.roll.read().unwrap()),
            transform_dirty: RwLock::new(*self.transform_dirty.read().unwrap()),
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
                    "x position" => {
                        if let Ok(value) = attribute.value.parse::<f32>() {
                            self.position.write().unwrap()[0] = value;
                            *self.transform_dirty.write().unwrap() = true;
                        }
                    },
                    "y position" => {
                        if let Ok(value) = attribute.value.parse::<f32>() {
                            self.position.write().unwrap()[1] = value;
                            *self.transform_dirty.write().unwrap() = true;
                        }
                    },
                    "z position" => {
                        if let Ok(value) = attribute.value.parse::<f32>() {
                            self.position.write().unwrap()[2] = value;
                            *self.transform_dirty.write().unwrap() = true;
                        }
                    },
                    "pitch (degrees)" => {
                        if let Ok(degrees) = attribute.value.parse::<f32>() {
                            *self.pitch.write().unwrap() = degrees.to_radians();
                            *self.transform_dirty.write().unwrap() = true;
                        }
                    },
                    "yaw (degrees)" => {
                        if let Ok(degrees) = attribute.value.parse::<f32>() {
                            *self.yaw.write().unwrap() = degrees.to_radians();
                            *self.transform_dirty.write().unwrap() = true;
                        }
                    },
                    "roll (degrees)" => {
                        if let Ok(degrees) = attribute.value.parse::<f32>() {
                            *self.roll.write().unwrap() = degrees.to_radians();
                            *self.transform_dirty.write().unwrap() = true;
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
        let position = *self.position.read().unwrap();
        let pitch = *self.pitch.read().unwrap();
        let yaw = *self.yaw.read().unwrap();
        let roll = *self.roll.read().unwrap();
        
        // Convert radians to degrees for better UI display
        let pitch_degrees = pitch.to_degrees();
        let yaw_degrees = yaw.to_degrees();
        let roll_degrees = roll.to_degrees();

        // Update existing SharedStrings in-place
        use slint::Model;
        for i in 0..ui.attributes.row_count() {
            if let Some(mut attr) = ui.attributes.row_data(i) {
                match attr.name.as_str() {
                    "x position" => attr.value = format!("{:.3}", position[0]).into(),
                    "y position" => attr.value = format!("{:.3}", position[1]).into(),
                    "z position" => attr.value = format!("{:.3}", position[2]).into(),
                    "pitch (degrees)" => attr.value = format!("{:.1}", pitch_degrees).into(),
                    "yaw (degrees)" => attr.value = format!("{:.1}", yaw_degrees).into(),
                    "roll (degrees)" => attr.value = format!("{:.1}", roll_degrees).into(),
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
        enum Field { Position, Pitch, Yaw }

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
                let mut position = None;
                let mut pitch = None;
                let mut yaw = None;
                
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Position => {
                            if position.is_some() {
                                return Err(de::Error::duplicate_field("position"));
                            }
                            position = Some(map.next_value()?);
                        }
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
                
                let position = position.ok_or_else(|| de::Error::missing_field("position"))?;
                let pitch = pitch.ok_or_else(|| de::Error::missing_field("pitch"))?;
                let yaw = yaw.ok_or_else(|| de::Error::missing_field("yaw"))?;
                
                // Create camera and set values
                let camera = Camera::new();
                *camera.position.write().unwrap() = position;
                *camera.pitch.write().unwrap() = pitch;
                *camera.yaw.write().unwrap() = yaw;
                *camera.transform_dirty.write().unwrap() = true;
                
                Ok(camera)
            }
        }

        const FIELDS: &'static [&'static str] = &["position", "pitch", "yaw"];
        deserializer.deserialize_struct("Camera", FIELDS, CameraVisitor)
    }
}
