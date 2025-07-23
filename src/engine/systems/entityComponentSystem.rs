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
//
// ------------------------------------------------------------------------
// Compile‑time requirements (Cargo.toml):
// once_cell = "1"
// uuid      = { version = "1", features = ["v4"] }
// paste     = "1"
//
// ------------------------------------------------------------------------
// USAGE EXAMPLE
// ------------------------------------------------------------------------
// ```rust
// let e = spawn();
// insert_many!(e, Transform(0.0, 0.0), Velocity(1.0, 0.0), Health(100));
//
// // iterate over every entity with Transform & Velocity
// query!((Transform, Velocity), |id, t, v| {
//     t.0 += v.0;
// });
//
// // operate on the *player* only
// if let Some(pid) = PLAYER_ENTITY_ID.read().unwrap().clone() {
//     query_by_id!(pid, (Transform, Health), |t, h| {
//         println!("player now at {:?} with {:?} HP", t, h);
//     });
// }
// ```

use once_cell::sync::Lazy;
use std::{ any::{ Any, TypeId }, collections::HashMap, sync::{ RwLock, RwLockWriteGuard } };
use uuid::Uuid;

/// Marker trait for data that can live in the ECS.
pub trait Component: Any + Send + Sync {}
impl<T: Any + Send + Sync> Component for T {}

pub type EntityId = String;

// ——————————————————————————————————————————————————————————— global state ————

pub static WORLD: Lazy<RwLock<World>> = Lazy::new(|| RwLock::new(World::default()));
pub fn world() -> RwLockWriteGuard<'static, World> {
    WORLD.write().expect("world lock")
}

/// Convenience function to spawn a new entity using the global world singleton
pub fn spawn() -> EntityId {
    world().spawn()
}

// ———————————————————————————————————————————————— internal structs ————

type ComponentMask = u64;

#[derive(Default)]
struct ComponentRegistry {
    next_bit: u8,
    bits: HashMap<TypeId, u8>,
}
impl ComponentRegistry {
    fn bit_for<T: Component>(&mut self) -> u8 {
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

pub struct World {
    stores: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
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

    pub fn insert<T: Component>(&mut self, id: &EntityId, comp: T) {
        let bit = self.registry.bit_for::<T>();
        let mask_bit = 1u64 << bit;
        let store = self.stores
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(Store::<T>::default()))
            .downcast_mut::<Store<T>>()
            .unwrap();
        store.insert(id, comp);
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
    fn get_component_for_entity<T: Component>(
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
                    .downcast_mut::<Store<T>>()
                    .unwrap();
                return store.get_mut(entity_id);
            }
        }
        None
    }

    // Read-only single component access
    pub fn get_component_readonly<T: Component>(
        &self,
        entity_id: &EntityId
    ) -> Option<&T> {
        // We need to check if the component type is already registered
        let type_id = TypeId::of::<T>();
        
        // Find the bit for this component type
        let bit = self.registry.bits.get(&type_id)?;
        let mask = 1u64 << bit;

        if let Some(&entity_mask) = self.meta.get(entity_id) {
            if (entity_mask & mask) == mask {
                let store = self.stores
                    .get(&type_id)?
                    .downcast_ref::<Store<T>>()
                    .unwrap();
                return store.0.get(entity_id);
            }
        }
        None
    }

}

// —————————————————————————————————————————— query traits ————————

/// Trait for types that can be used as query parameters in the ECS.
/// This enables variadic queries by implementing the trait for tuples recursively.
pub trait QueryParam {
    /// The type returned when fetching components for this query parameter.
    type Output<'w>;
    
    /// Compute the combined bitmask for all component types in this query.
    fn mask_bits(registry: &mut ComponentRegistry) -> u64;
    
    /// Fetch the components for a given entity, returning None if any required component is missing.
    fn fetch_components<'w>(world: &'w mut World, entity: &EntityId) -> Option<Self::Output<'w>>;
}

// Implementation for single mutable component references
impl<T: Component> QueryParam for &mut T {
    type Output<'w> = &'w mut T;
    
    fn mask_bits(registry: &mut ComponentRegistry) -> u64 {
        1u64 << registry.bit_for::<T>()
    }
    
    fn fetch_components<'w>(world: &'w mut World, entity: &EntityId) -> Option<Self::Output<'w>> {
        world.get_component_for_entity::<T>(entity)
    }
}

// Implementation for single immutable component references
impl<T: Component> QueryParam for &T {
    type Output<'w> = &'w T;
    
    fn mask_bits(registry: &mut ComponentRegistry) -> u64 {
        1u64 << registry.bit_for::<T>()
    }
    
    fn fetch_components<'w>(world: &'w mut World, entity: &EntityId) -> Option<Self::Output<'w>> {
        // For immutable access, we need to work around the mutable world reference
        // This is a limitation of the current design - we'll use unsafe to get immutable access
        unsafe {
            let world_ptr = world as *const World;
            (*world_ptr).get_component_readonly::<T>(entity)
        }
    }
}

// Implementation for single-element tuples
impl<Q: QueryParam> QueryParam for (Q,) {
    type Output<'w> = (Q::Output<'w>,);
    
    fn mask_bits(registry: &mut ComponentRegistry) -> u64 {
        Q::mask_bits(registry)
    }
    
    fn fetch_components<'w>(world: &'w mut World, entity: &EntityId) -> Option<Self::Output<'w>> {
        Some((Q::fetch_components(world, entity)?,))
    }
}

// Recursive implementation for tuples - this enables arbitrary-length queries
impl<Q1: QueryParam, Q2: QueryParam> QueryParam for (Q1, Q2) {
    type Output<'w> = (Q1::Output<'w>, Q2::Output<'w>);
    
    fn mask_bits(registry: &mut ComponentRegistry) -> u64 {
        Q1::mask_bits(registry) | Q2::mask_bits(registry)
    }
    
    fn fetch_components<'w>(world: &'w mut World, entity: &EntityId) -> Option<Self::Output<'w>> {
        // We need to handle the borrowing carefully here
        // This is complex due to Rust's borrowing rules, so we'll use unsafe
        unsafe {
            let world_ptr = world as *mut World;
            let comp1 = Q1::fetch_components(&mut *world_ptr, entity)?;
            let comp2 = Q2::fetch_components(&mut *world_ptr, entity)?;
            Some((comp1, comp2))
        }
    }
}

// —————————————————————————————————————————— dynamic traits ————————

pub trait Insertable {
    fn insert_into(self: Box<Self>, w: &mut World, id: &EntityId);
}
impl<T: Component> Insertable for T {
    fn insert_into(self: Box<Self>, w: &mut World, id: &EntityId) {
        w.insert(id, *self)
    }
}

// —————————————————————————————————————————— helper macros ————————

#[macro_export]
macro_rules! insert_many {
    ($entity:expr $(, $comp:expr)+ $(,)?) => {
        {
        use std::boxed::Box; use crate::index::engine::systems::entityComponentSystem::Insertable;
        let mut v: Vec<Box<dyn crate::index::engine::systems::entityComponentSystem::Insertable>> = Vec::new();
        $( v.push(Box::new($comp)); )+
        crate::index::engine::systems::entityComponentSystem::world().insert_dyn(&$entity, v);
        }
    };
}

// Simple macro implementation without paste crate

impl World {
    /// Generic query method that works with any number of components using the QueryParam trait.
    /// This replaces all the queryN methods with a single generic implementation.
    pub fn query<Q, F>(&mut self, f: F)
    where
        Q: QueryParam,
        F: for<'w> FnMut(&EntityId, Q::Output<'w>),
    {
        // 1. Compute the combined mask for the query's component types
        let mask = Q::mask_bits(&mut self.registry);
        
        // 2. Gather all entities that have at least those components (entity_mask & mask == mask)
        let entities: Vec<EntityId> = self.meta
            .iter()
            .filter(|(_, &entity_mask)| (entity_mask & mask) == mask)
            .map(|(id, _)| id.clone())
            .collect();
        
        // 3. For each such entity, fetch the actual components and call the user closure
        let mut f = f;
        for id in entities {
            if let Some(components) = Q::fetch_components(self, &id) {
                f(&id, components);
            }
        }
    }

    /// Generic query method for a single entity using the QueryParam trait.
    /// This replaces all the query_by_idN methods with a single generic implementation.
    pub fn query_one<Q, F>(&mut self, entity: &EntityId, f: F)
    where
        Q: QueryParam,
        F: for<'w> FnOnce(Q::Output<'w>),
    {
        // Ensure the entity has all components in the query mask
        let mask = Q::mask_bits(&mut self.registry);
        if let Some(&entity_mask) = self.meta.get(entity) {
            if (entity_mask & mask) == mask {
                if let Some(comps) = Q::fetch_components(self, entity) {
                    f(comps);
                }
            }
        }
    }
}

// convenience front‑end macros --------------------------------------------

#[macro_export]
macro_rules! query {
    // Single component
    (($c1:ty), | $id:ident, $a1:ident | $body:block) => {
        crate::index::engine::systems::entityComponentSystem::world().query::<&mut $c1, _>(|$id, $a1| $body)
    };
    // Two components
    (($c1:ty, $c2:ty), | $id:ident, $a1:ident, $a2:ident | $body:block) => {
        crate::index::engine::systems::entityComponentSystem::world().query::<(&mut $c1, &mut $c2), _>(|$id, ($a1, $a2)| $body)
    };
    // Three components
    (($c1:ty, $c2:ty, $c3:ty), | $id:ident, $a1:ident, $a2:ident, $a3:ident | $body:block) => {
        crate::index::engine::systems::entityComponentSystem::world().query::<(&mut $c1, (&mut $c2, &mut $c3)), _>(|$id, ($a1, ($a2, $a3))| $body)
    };
    // Four components
    (($c1:ty, $c2:ty, $c3:ty, $c4:ty), | $id:ident, $a1:ident, $a2:ident, $a3:ident, $a4:ident | $body:block) => {
        crate::index::engine::systems::entityComponentSystem::world().query::<(&mut $c1, (&mut $c2, (&mut $c3, &mut $c4))), _>(|$id, ($a1, ($a2, ($a3, $a4)))| $body)
    };
    // Five components
    (($c1:ty, $c2:ty, $c3:ty, $c4:ty, $c5:ty), | $id:ident, $a1:ident, $a2:ident, $a3:ident, $a4:ident, $a5:ident | $body:block) => {
        crate::index::engine::systems::entityComponentSystem::world().query::<(&mut $c1, (&mut $c2, (&mut $c3, (&mut $c4, &mut $c5)))), _>(|$id, ($a1, ($a2, ($a3, ($a4, $a5))))| $body)
    };
}

#[macro_export]
macro_rules! query_by_id {
    // Single component
    ($eid:expr, ($c1:ty), | $a1:ident | $body:block) => {
        crate::index::engine::systems::entityComponentSystem::world().query_one::<&mut $c1, _>(&$eid, |$a1| $body)
    };
    // Two components
    ($eid:expr, ($c1:ty, $c2:ty), | $a1:ident, $a2:ident | $body:block) => {
        crate::index::engine::systems::entityComponentSystem::world().query_one::<(&mut $c1, &mut $c2), _>(&$eid, |($a1, $a2)| $body)
    };
    // Three components
    ($eid:expr, ($c1:ty, $c2:ty, $c3:ty), | $a1:ident, $a2:ident, $a3:ident | $body:block) => {
        crate::index::engine::systems::entityComponentSystem::world().query_one::<(&mut $c1, (&mut $c2, &mut $c3)), _>(&$eid, |($a1, ($a2, $a3))| $body)
    };
    // Four components
    ($eid:expr, ($c1:ty, $c2:ty, $c3:ty, $c4:ty), | $a1:ident, $a2:ident, $a3:ident, $a4:ident | $body:block) => {
        crate::index::engine::systems::entityComponentSystem::world().query_one::<(&mut $c1, (&mut $c2, (&mut $c3, &mut $c4))), _>(&$eid, |($a1, ($a2, ($a3, $a4)))| $body)
    };
    // Five components
    ($eid:expr, ($c1:ty, $c2:ty, $c3:ty, $c4:ty, $c5:ty), | $a1:ident, $a2:ident, $a3:ident, $a4:ident, $a5:ident | $body:block) => {
        crate::index::engine::systems::entityComponentSystem::world().query_one::<(&mut $c1, (&mut $c2, (&mut $c3, (&mut $c4, &mut $c5)))), _>(&$eid, |($a1, ($a2, ($a3, ($a4, $a5))))| $body)
    };
}

// New get_query_by_id! macro - returns read-only components instead of using callback
#[macro_export]
macro_rules! get_query_by_id {
    ($eid:expr, ($c1:ty)) => {
        {
            let world = crate::index::engine::systems::entityComponentSystem::WORLD.read().expect("world lock");
            world.get_component_readonly::<$c1>(&$eid).cloned()
        }
    };
}
