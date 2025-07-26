// src/world.rs
// Minimal ECS with **string‑based entity IDs**, a global singleton `World`,
// variadic *insert* **and now variadic *query***.
//
//   • `insert_many!(id, C1(..), C2(..), …)` – insert multiple components atomically.
//   • `query!((A, B, C), |id, a, b, c| { … })` – iterate over **all**
//     entities that own *every* listed component.
//   • `query_by_id!(id, (A, B, C), |a, b, c| { … })` – borrow those
//     components for **one** entity.
//
// These procedures are implemented as *variadic macros* that expand to the
// corresponding `queryN` / `withN` specialisations and use the global world
// singleton directly. We ship ready‑made implementations up to **ten**
// components – extend if you need more.

use std::{ any::{ Any, TypeId }, collections::HashMap, cell::RefCell };
use uuid::Uuid;

// Import Slint-generated types directly
// These will be available after slint::include_modules!() is called
// We'll use them through the crate root imports

pub trait Component: Any {
    fn apply_ui(&mut self, component_ui: &crate::ComponentUI);                    // Apply UI changes to component
    fn update_component_ui(&mut self, entity_id: &str);                          // Update SharedStrings when component changes
    fn get_component_ui(&self) -> std::rc::Rc<std::cell::RefCell<crate::ComponentUI>>; // Return direct reference for live updates
}

// Dynamic store trait for collecting ComponentUI without hardcoding component types
pub trait StoreDyn: Any {
    /// If the given entity has a component in this store, return its UI representation.
    fn get_component_ui_for_entity(&self, id: &EntityId) -> Option<crate::ComponentUI>;
    
    /// Apply ComponentUI changes to the component in this store for the given entity.
    fn apply_component_ui(&mut self, id: &EntityId, ui_data: &crate::ComponentUI);
    
    /// Provide a way to get a `&dyn Any` for downcasting to concrete store type if needed.
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub type EntityId = String;

// ——————————————————————————————————————————————————————————— global state ————

thread_local! {
    pub static WORLD: RefCell<World> = RefCell::new(World::default());
}

/// Convenience function to spawn a new entity using the global world singleton
pub fn spawn() -> EntityId {
    WORLD.with(|w| {
        let mut world = w.borrow_mut();
        world.spawn()
    })
}

// ———————————————————————————————————————————————— internal structs ————

type ComponentMask = u64;

#[derive(Default)]
pub struct ComponentRegistry {
    next_bit: u8,
    bits: HashMap<TypeId, u8>,
}
impl ComponentRegistry {
    fn bit_for<T: Component + Clone>(&mut self) -> u8 {
        *self.bits.entry(TypeId::of::<T>()).or_insert_with(|| {
            let b = self.next_bit;
            assert!(b < 64);
            self.next_bit += 1;
            b
        })
    }
}

struct Store<T: Component>(HashMap<EntityId, T>);
impl<T: Component> Default for Store<T> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}
impl<T: Component> Store<T> {
    fn insert(&mut self, id: &EntityId, v: T) {
        self.0.insert(id.clone(), v);
    }
    fn get_mut(&mut self, id: &EntityId) -> Option<&mut T> {
        self.0.get_mut(id)
    }
}

// Implement StoreDyn for Store<T> to enable dynamic ComponentUI collection
impl<T: Component + Clone + 'static> StoreDyn for Store<T> {
    fn get_component_ui_for_entity(&self, id: &EntityId) -> Option<crate::ComponentUI> {
        // If this entity has a T component, get its UI state
        self.0.get(id).map(|component| {
            // Get the ComponentUI (Rc<RefCell<...>>), borrow it, and clone the inner data
            component.get_component_ui().borrow().clone()
        })
    }

    fn apply_component_ui(&mut self, id: &EntityId, ui_data: &crate::ComponentUI) {
        // If this entity has a T component, apply the UI changes to it
        if let Some(component) = self.0.get_mut(id) {
            component.apply_ui(ui_data);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct World {
    stores: HashMap<TypeId, Box<dyn StoreDyn>>,
    meta: HashMap<EntityId, ComponentMask>,
    registry: ComponentRegistry,
}
impl Default for World {
    fn default() -> Self {
        Self {
            stores: HashMap::new(),
            meta: HashMap::new(),
            registry: ComponentRegistry::default(),
        }
    }
}

// ——————————————————————————————————————————————————————————— API ————

impl World {
    pub fn spawn(&mut self) -> EntityId {
        let id = Uuid::new_v4().to_string();
        self.meta.insert(id.clone(), 0);
        id
    }

    pub fn insert<T: Component + Clone + 'static>(&mut self, id: &EntityId, comp: T) {
        let bit = self.registry.bit_for::<T>();
        let mask_bit = 1u64 << bit;
        let type_id = TypeId::of::<T>();

        // Insert into typed store (as trait object)
        let store_dyn = self.stores
            .entry(type_id)
            .or_insert_with(|| {
                // Create a new Store<T> and cast to Box<dyn StoreDyn>
                Box::new(Store::<T>::default()) as Box<dyn StoreDyn>
            });
        // Now `store_dyn` is a Box<dyn StoreDyn> but we know it's actually a Store<T>
        // Downcast it to insert the component
        store_dyn.as_any_mut()
            .downcast_mut::<Store<T>>()
            .unwrap()
            .insert(id, comp.clone());

        // Update entity mask
        self.meta
            .entry(id.clone())
            .and_modify(|m| {
                *m |= mask_bit;
            })
            .or_insert(mask_bit);
    }

    pub fn insert_dyn(&mut self, id: &EntityId, comps: Vec<Box<dyn Insertable>>) {
        for c in comps {
            c.insert_into(self, id);
        }
    }

    // Single component access for specific entity
    fn get_component_for_entity<T: Component + Clone>(
        &mut self,
        entity_id: &EntityId
    ) -> Option<&mut T> {
        let bit = self.registry.bit_for::<T>();
        let mask = 1u64 << bit;

        if let Some(&entity_mask) = self.meta.get(entity_id) {
            if (entity_mask & mask) == mask {
                let store = self.stores
                    .get_mut(&TypeId::of::<T>())
                    .unwrap()
                    .as_any_mut()
                    .downcast_mut::<Store<T>>()
                    .unwrap();
                return store.get_mut(entity_id);
            }
        }
        None
    }


    // Read-only single component access
    pub fn get_component_readonly<T: Component>(&self, entity_id: &EntityId) -> Option<&T> {
        // We need to check if the component type is already registered
        let type_id = TypeId::of::<T>();

        // Find the bit for this component type
        let bit = self.registry.bits.get(&type_id)?;
        let mask = 1u64 << bit;

        if let Some(&entity_mask) = self.meta.get(entity_id) {
            if (entity_mask & mask) == mask {
                let store = self.stores.get(&type_id)?.as_any().downcast_ref::<Store<T>>().unwrap();
                return store.0.get(entity_id);
            }
        }
        None
    }

    // Query methods for 1-5 components
    #[allow(dead_code)]
    pub fn query1<F, C1: Component + Clone>(&mut self, mut f: F) where F: FnMut(&EntityId, &mut C1) {
        let bit1 = self.registry.bit_for::<C1>();
        let mask = 1u64 << bit1;
        let entities: Vec<EntityId> = self.meta
            .iter()
            .filter(|(_, &m)| (m & mask) == mask)
            .map(|(id, _)| id.clone())
            .collect();
        for eid in entities {
            if let Some(c1) = self.get_component_for_entity::<C1>(&eid) {
                f(&eid, c1);
            }
        }
    }

    pub fn query2<F, C1: Component + Clone, C2: Component + Clone>(&mut self, mut f: F)
        where F: FnMut(&EntityId, &mut C1, &mut C2)
    {
        let bit1 = self.registry.bit_for::<C1>();
        let bit2 = self.registry.bit_for::<C2>();
        let mask = (1u64 << bit1) | (1u64 << bit2);
        let entities: Vec<EntityId> = self.meta
            .iter()
            .filter(|(_, &m)| (m & mask) == mask)
            .map(|(id, _)| id.clone())
            .collect();
        for eid in entities {
            unsafe {
                let world_ptr = self as *mut World;
                if
                    let (Some(c1), Some(c2)) = (
                        (*world_ptr).get_component_for_entity::<C1>(&eid),
                        (*world_ptr).get_component_for_entity::<C2>(&eid),
                    )
                {
                    f(&eid, c1, c2);
                }
            }
        }
    }

    pub fn query_by_id1<F, C1: Component + Clone>(&mut self, entity: &EntityId, mut f: F)
        where F: FnMut(&mut C1)
    {
        if let Some(c1) = self.get_component_for_entity::<C1>(entity) {
            f(c1);
        }
    }

    #[allow(dead_code)]
    pub fn query_by_id2<F, C1: Component + Clone, C2: Component + Clone>(
        &mut self,
        entity: &EntityId,
        mut f: F
    )
        where F: FnMut(&mut C1, &mut C2)
    {
        unsafe {
            let world_ptr = self as *mut World;
            if
                let (Some(c1), Some(c2)) = (
                    (*world_ptr).get_component_for_entity::<C1>(entity),
                    (*world_ptr).get_component_for_entity::<C2>(entity),
                )
            {
                f(c1, c2);
            }
        }
    }

    /// Get all entities that have a specific component type
    pub fn query_get_all<T: Component>(&self) -> Vec<(EntityId, T)> where T: Clone {
        let type_id = TypeId::of::<T>();

        // Find the bit for this component type
        let bit = match self.registry.bits.get(&type_id) {
            Some(bit) => *bit,
            None => {
                return Vec::new();
            } // Component type not registered
        };

        let mask = 1u64 << bit;
        let mut results = Vec::new();

        // Get all entities that have this component
        for (entity_id, &entity_mask) in &self.meta {
            if (entity_mask & mask) == mask {
                if let Some(store) = self.stores.get(&type_id) {
                    if let Some(store) = store.as_any().downcast_ref::<Store<T>>() {
                        if let Some(component) = store.0.get(entity_id) {
                            results.push((entity_id.clone(), component.clone()));
                        }
                    }
                }
            }
        }

        results
    }

    /// Get all entity IDs that have a specific component type
    pub fn query_get_all_ids<T: Component>(&self) -> Vec<EntityId> {
        let type_id = TypeId::of::<T>();

        // Find the bit for this component type
        let bit = match self.registry.bits.get(&type_id) {
            Some(bit) => *bit,
            None => {
                return Vec::new();
            } // Component type not registered
        };

        let mask = 1u64 << bit;
        let mut results = Vec::new();

        // Get all entities that have this component
        for (entity_id, &entity_mask) in &self.meta {
            if (entity_mask & mask) == mask {
                results.push(entity_id.clone());
            }
        }

        results
    }

    /// Get all components for a specific entity as ComponentUI - Dynamic implementation using StoreDyn
    pub fn get_all_components_ui_for_entity(&self, entity_id: &EntityId) -> Vec<crate::ComponentUI> {
        let mut ui_components = Vec::new();
        if let Some(_mask) = self.meta.get(entity_id) {
            // Iterate over all component stores and collect UI for this entity
            for store in self.stores.values() {
                if let Some(component_ui) = store.get_component_ui_for_entity(entity_id) {
                    ui_components.push(component_ui);
                }
            }
        }
        ui_components
    }

    /// Apply ComponentUI changes to a component dynamically using TypeId lookup
    pub fn apply_component_ui_by_type(&mut self, entity_id: &EntityId, type_id: &TypeId, ui_data: &crate::ComponentUI) -> bool {
        if let Some(store) = self.stores.get_mut(type_id) {
            store.apply_component_ui(entity_id, ui_data);
            true
        } else {
            false
        }
    }
}

// —————————————————————————————————————————— dynamic traits ————————

pub trait Insertable {
    fn insert_into(self: Box<Self>, w: &mut World, id: &EntityId);
}
impl<T: Component + Clone> Insertable for T {
    fn insert_into(self: Box<Self>, w: &mut World, id: &EntityId) {
        w.insert(id, *self)
    }
}

// —————————————————————————————————————————— helper macros ————————

#[macro_export]
macro_rules! insert_many {
    ($entity:expr $(, $comp:expr)+ $(,)?) => {
        {
        use std::boxed::Box;
        let mut v: Vec<Box<dyn crate::index::engine::systems::entity_component_system::Insertable>> = Vec::new();
        $( v.push(Box::new($comp)); )+
        crate::index::engine::systems::entity_component_system::WORLD.with(|w| {
            let mut world = w.borrow_mut();
            world.insert_dyn(&$entity, v);
        });
        }
    };
}

#[macro_export]
macro_rules! query {
    // Single component
    (($c1:ty), | $id:ident, $a1:ident | $body:block) => {
        crate::index::engine::systems::entity_component_system::WORLD.with(|w| {
            let mut world = w.borrow_mut();
            world.query1::<_, $c1>(|$id, $a1| $body)
        })
    };
    // Two components
    (($c1:ty, $c2:ty), | $id:ident, $a1:ident, $a2:ident | $body:block) => {
        crate::index::engine::systems::entity_component_system::WORLD.with(|w| {
            let mut world = w.borrow_mut();
            world.query2::<_, $c1, $c2>(|$id, $a1, $a2| $body)
        })
    };
}

#[macro_export]
macro_rules! query_by_id {
    // Single component
    ($eid:expr, ($c1:ty), | $a1:ident | $body:block) => {
        crate::index::engine::systems::entity_component_system::WORLD.with(|w| {
            let mut world = w.borrow_mut();
            world.query_by_id1::<_, $c1>(&$eid, |$a1| $body)
        })
    };
    // Two components
    ($eid:expr, ($c1:ty, $c2:ty), | $a1:ident, $a2:ident | $body:block) => {
        crate::index::engine::systems::entity_component_system::WORLD.with(|w| {
            let mut world = w.borrow_mut();
            world.query_by_id2::<_, $c1, $c2>(&$eid, |$a1, $a2| $body)
        })
    };
}

// New get_query_by_id! macro - returns read-only components instead of using callback
#[macro_export]
macro_rules! get_query_by_id {
    ($eid:expr, ($c1:ty)) => {
        {
            crate::index::engine::systems::entity_component_system::WORLD.with(|w| {
                let world = w.borrow();
                world.get_component_readonly::<$c1>(&$eid).cloned()
            })
        }
    };
}

// New query_get_all! macro - returns all entities with a specific component
#[macro_export]
macro_rules! query_get_all {
    ($c1:ty) => {
        {
            crate::index::engine::systems::entity_component_system::WORLD.with(|w| {
                let world = w.borrow();
                world.query_get_all::<$c1>()
            })
        }
    };
}

// New query_get_all_ids! macro - returns all entity IDs with a specific component
#[macro_export]
macro_rules! query_get_all_ids {
    ($c1:ty) => {
        {
            crate::index::engine::systems::entity_component_system::WORLD.with(|w| {
                let world = w.borrow();
                world.query_get_all_ids::<$c1>()
            })
        }
    };
}

#[macro_export]
macro_rules! get_all_components_by_id {
    ($eid:expr) => {
        {
            crate::index::engine::systems::entity_component_system::WORLD.with(|w| {
                let world = w.borrow();
                world.get_all_components_ui_for_entity(&$eid)
            })
        }
    };
}
