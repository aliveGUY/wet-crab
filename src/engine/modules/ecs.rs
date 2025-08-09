use std::collections::HashMap;
use std::sync::RwLock;
use once_cell::sync::Lazy;
use serde::{ Serialize, Deserialize };
use uuid::Uuid;

// Import all component types
use crate::index::engine::components::{
    rigid_body::RigidBody,
    AnimatedObject3DComponent as AnimatedObject3D,
    CameraComponent as Camera,
    Collider,
    Metadata,
    Shape,
    StaticObject3DComponent as StaticObject3D,
    Transform,
};

pub type EntityId = String;

// ——————————————————————————————————————————————————————————— Component Enum ————

/// Main component enum that wraps all component types with serde type tagging
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Component {
    Transform(Transform),
    Metadata(Metadata),
    Camera(Camera),
    Collider(Collider),
    StaticObject3D(StaticObject3D),
    AnimatedObject3D(AnimatedObject3D),
    Shape(Shape),
    RigidBody(RigidBody),
}

// ——————————————————————————————————————————————————————————— Global Singleton ————

/// Global component map singleton - HashMap<EntityId, Vec<Component>>
static COMPONENT_MAP: Lazy<RwLock<HashMap<String, Vec<Component>>>> = Lazy::new(||
    RwLock::new(HashMap::new())
);

// ——————————————————————————————————————————————————————————— Core API ————

/// Spawn a new entity and return its ID
pub fn spawn() -> EntityId {
    let id = Uuid::new_v4().to_string();
    let mut map = COMPONENT_MAP.write().unwrap();
    map.insert(id.clone(), Vec::new());
    id
}

/// Insert a component into an entity
pub fn insert<T>(entity_id: &EntityId, component: T) where T: Into<Component> + Clone {
    let mut map = COMPONENT_MAP.write().unwrap();
    if let Some(components) = map.get_mut(entity_id) {
        // Remove existing component of the same type if it exists
        let new_component = component.into();
        components.retain(|c| std::mem::discriminant(c) != std::mem::discriminant(&new_component));
        components.push(new_component);
    }
}

/// Get a component from an entity (read-only)
pub fn get_component<T>(entity_id: &EntityId) -> Option<T> where T: Clone, Component: TryInto<T> {
    let map = COMPONENT_MAP.read().unwrap();
    if let Some(components) = map.get(entity_id) {
        for component in components {
            if let Ok(typed_component) = component.clone().try_into() {
                return Some(typed_component);
            }
        }
    }
    None
}

/// Get a mutable reference to a component (requires write lock)
pub fn get_component_mut<T, F, R>(entity_id: &EntityId, f: F) -> Option<R>
    where T: Clone, Component: TryInto<T> + From<T>, F: FnOnce(&mut T) -> R
{
    let mut map = COMPONENT_MAP.write().unwrap();
    if let Some(components) = map.get_mut(entity_id) {
        for component in components.iter_mut() {
            if let Ok(mut typed_component) = component.clone().try_into() {
                let result = f(&mut typed_component);
                *component = typed_component.into();
                return Some(result);
            }
        }
    }
    None
}

/// Query all entities with a specific component type
pub fn query_all<T>() -> Vec<(EntityId, T)> where T: Clone, Component: TryInto<T> {
    let map = COMPONENT_MAP.read().unwrap();
    let mut results = Vec::new();

    for (entity_id, components) in map.iter() {
        for component in components {
            if let Ok(typed_component) = component.clone().try_into() {
                results.push((entity_id.clone(), typed_component));
                break; // Only one component of each type per entity
            }
        }
    }

    results
}

/// Query all entities with two specific component types
pub fn query_all2<T1, T2>() -> Vec<(EntityId, T1, T2)>
    where T1: Clone, T2: Clone, Component: TryInto<T1> + TryInto<T2>
{
    let map = COMPONENT_MAP.read().unwrap();
    let mut results = Vec::new();

    for (entity_id, components) in map.iter() {
        let mut comp1: Option<T1> = None;
        let mut comp2: Option<T2> = None;

        for component in components {
            if comp1.is_none() {
                if let Ok(typed_component) = component.clone().try_into() {
                    comp1 = Some(typed_component);
                    continue;
                }
            }
            if comp2.is_none() {
                if let Ok(typed_component) = component.clone().try_into() {
                    comp2 = Some(typed_component);
                    continue;
                }
            }
        }

        if let (Some(c1), Some(c2)) = (comp1, comp2) {
            results.push((entity_id.clone(), c1, c2));
        }
    }

    results
}

/// Query all entities with three specific component types
pub fn query_all3<T1, T2, T3>() -> Vec<(EntityId, T1, T2, T3)>
    where T1: Clone, T2: Clone, T3: Clone, Component: TryInto<T1> + TryInto<T2> + TryInto<T3>
{
    let map = COMPONENT_MAP.read().unwrap();
    let mut results = Vec::new();

    for (entity_id, components) in map.iter() {
        let mut comp1: Option<T1> = None;
        let mut comp2: Option<T2> = None;
        let mut comp3: Option<T3> = None;

        for component in components {
            if comp1.is_none() {
                if let Ok(typed_component) = component.clone().try_into() {
                    comp1 = Some(typed_component);
                    continue;
                }
            }
            if comp2.is_none() {
                if let Ok(typed_component) = component.clone().try_into() {
                    comp2 = Some(typed_component);
                    continue;
                }
            }
            if comp3.is_none() {
                if let Ok(typed_component) = component.clone().try_into() {
                    comp3 = Some(typed_component);
                    continue;
                }
            }
        }

        if let (Some(c1), Some(c2), Some(c3)) = (comp1, comp2, comp3) {
            results.push((entity_id.clone(), c1, c2, c3));
        }
    }

    results
}

/// Get all entity IDs that have a specific component type
pub fn query_get_all_ids<T>() -> Vec<EntityId> where Component: TryInto<T> {
    let map = COMPONENT_MAP.read().unwrap();
    let mut results = Vec::new();

    for (entity_id, components) in map.iter() {
        for component in components {
            if component.clone().try_into().is_ok() {
                results.push(entity_id.clone());
                break;
            }
        }
    }

    results
}

/// Copy an entity with all its components to a new entity
pub fn copy_entity(source_entity_id: &EntityId) -> Option<EntityId> {
    let mut map = COMPONENT_MAP.write().unwrap();

    if let Some(source_components) = map.get(source_entity_id).cloned() {
        let new_entity_id = Uuid::new_v4().to_string();
        map.insert(new_entity_id.clone(), source_components);
        Some(new_entity_id)
    } else {
        None
    }
}

/// Delete an entity and all its components
pub fn delete_entity(entity_id: &EntityId) -> bool {
    let mut map = COMPONENT_MAP.write().unwrap();
    map.remove(entity_id).is_some()
}

/// Get all entities and their component counts (for debugging/serialization)
pub fn get_all_entities() -> Vec<(EntityId, usize)> {
    let map = COMPONENT_MAP.read().unwrap();
    map.iter()
        .map(|(id, components)| (id.clone(), components.len()))
        .collect()
}

/// Get all components for a specific entity
pub fn get_all_components(entity_id: &EntityId) -> Vec<Component> {
    let map = COMPONENT_MAP.read().unwrap();
    map.get(entity_id).cloned().unwrap_or_default()
}

/// Serialize the entire component map to JSON
pub fn serialize_to_json() -> Result<String, serde_json::Error> {
    let map = COMPONENT_MAP.read().unwrap();
    serde_json::to_string_pretty(&*map)
}

/// Deserialize the entire component map from JSON
pub fn deserialize_from_json(json: &str) -> Result<(), serde_json::Error> {
    let new_map: HashMap<String, Vec<Component>> = serde_json::from_str(json)?;
    let mut map = COMPONENT_MAP.write().unwrap();
    *map = new_map;
    Ok(())
}

/// Clear all entities and components
pub fn clear_world() {
    let mut map = COMPONENT_MAP.write().unwrap();
    map.clear();
}

// ——————————————————————————————————————————————————————————— Conversion Traits ————

// Implement Into<Component> for all component types
impl From<Transform> for Component {
    fn from(t: Transform) -> Self {
        Component::Transform(t)
    }
}

impl From<Metadata> for Component {
    fn from(m: Metadata) -> Self {
        Component::Metadata(m)
    }
}

impl From<Camera> for Component {
    fn from(c: Camera) -> Self {
        Component::Camera(c)
    }
}

impl From<Collider> for Component {
    fn from(c: Collider) -> Self {
        Component::Collider(c)
    }
}

impl From<StaticObject3D> for Component {
    fn from(s: StaticObject3D) -> Self {
        Component::StaticObject3D(s)
    }
}

impl From<AnimatedObject3D> for Component {
    fn from(a: AnimatedObject3D) -> Self {
        Component::AnimatedObject3D(a)
    }
}

impl From<Shape> for Component {
    fn from(s: Shape) -> Self {
        Component::Shape(s)
    }
}

impl From<RigidBody> for Component {
    fn from(s: RigidBody) -> Self {
        Component::RigidBody(s)
    }
}

// Implement TryInto<T> for Component to extract specific types
impl TryInto<Transform> for Component {
    type Error = ();

    fn try_into(self) -> Result<Transform, Self::Error> {
        match self {
            Component::Transform(t) => Ok(t),
            _ => Err(()),
        }
    }
}

impl TryInto<Metadata> for Component {
    type Error = ();

    fn try_into(self) -> Result<Metadata, Self::Error> {
        match self {
            Component::Metadata(m) => Ok(m),
            _ => Err(()),
        }
    }
}

impl TryInto<Camera> for Component {
    type Error = ();

    fn try_into(self) -> Result<Camera, Self::Error> {
        match self {
            Component::Camera(c) => Ok(c),
            _ => Err(()),
        }
    }
}

impl TryInto<Collider> for Component {
    type Error = ();

    fn try_into(self) -> Result<Collider, Self::Error> {
        match self {
            Component::Collider(c) => Ok(c),
            _ => Err(()),
        }
    }
}

impl TryInto<StaticObject3D> for Component {
    type Error = ();

    fn try_into(self) -> Result<StaticObject3D, Self::Error> {
        match self {
            Component::StaticObject3D(s) => Ok(s),
            _ => Err(()),
        }
    }
}

impl TryInto<AnimatedObject3D> for Component {
    type Error = ();

    fn try_into(self) -> Result<AnimatedObject3D, Self::Error> {
        match self {
            Component::AnimatedObject3D(a) => Ok(a),
            _ => Err(()),
        }
    }
}

impl TryInto<Shape> for Component {
    type Error = ();

    fn try_into(self) -> Result<Shape, Self::Error> {
        match self {
            Component::Shape(s) => Ok(s),
            _ => Err(()),
        }
    }
}

// ——————————————————————————————————————————————————————————— Compatibility Layer ————

/// Legacy World struct for compatibility (now just a wrapper)
pub struct World;

impl World {
    pub fn spawn(&mut self) -> EntityId {
        spawn()
    }

    pub fn insert<T>(&mut self, entity_id: &EntityId, component: T) where T: Into<Component> + Clone {
        insert(entity_id, component);
    }

    pub fn get_component_readonly<T>(&self, entity_id: &EntityId) -> Option<T>
        where T: Clone, Component: TryInto<T>
    {
        get_component(entity_id)
    }

    pub fn query_get_all<T>(&self) -> Vec<(EntityId, T)> where T: Clone, Component: TryInto<T> {
        query_all()
    }

    pub fn query_get_all2<T1, T2>(&self) -> Vec<(EntityId, T1, T2)>
        where T1: Clone, T2: Clone, Component: TryInto<T1> + TryInto<T2>
    {
        query_all2()
    }

    pub fn query_get_all3<T1, T2, T3>(&self) -> Vec<(EntityId, T1, T2, T3)>
        where T1: Clone, T2: Clone, T3: Clone, Component: TryInto<T1> + TryInto<T2> + TryInto<T3>
    {
        query_all3()
    }

    pub fn query_get_all_ids<T>(&self) -> Vec<EntityId> where Component: TryInto<T> {
        query_get_all_ids::<T>()
    }

    pub fn copy_entity(&mut self, source_entity_id: &EntityId) -> Option<EntityId> {
        copy_entity(source_entity_id)
    }

    pub fn delete_entity(&mut self, entity_id: &EntityId) -> bool {
        delete_entity(entity_id)
    }

    pub fn get_all_entities(&self) -> Vec<(EntityId, usize)> {
        get_all_entities()
    }
}

impl Default for World {
    fn default() -> Self {
        World
    }
}

// Legacy WORLD thread-local for compatibility
thread_local! {
    pub static WORLD: std::cell::RefCell<World> = const { std::cell::RefCell::new(World) };
}

// ——————————————————————————————————————————————————————————— New System Functions ————

// The new system provides direct function calls instead of macros to avoid conflicts
// Use these functions directly:
// - spawn() -> EntityId
// - insert(entity_id, component)
// - get_component::<T>(entity_id) -> Option<T>
// - query_all::<T>() -> Vec<(EntityId, T)>
// - query_all2::<T1, T2>() -> Vec<(EntityId, T1, T2)>
// - query_all3::<T1, T2, T3>() -> Vec<(EntityId, T1, T2, T3)>
// - copy_entity(source_id) -> Option<EntityId>
// - delete_entity(entity_id) -> bool
// - serialize_to_json() -> Result<String, serde_json::Error>
// - deserialize_from_json(json) -> Result<(), serde_json::Error>
