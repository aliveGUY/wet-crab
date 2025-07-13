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

static WORLD: Lazy<RwLock<World>> = Lazy::new(|| RwLock::new(World::default()));
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

    // Helper method to get component stores for specific entity
    fn get_component_stores_for_entity<T1: Component, T2: Component>(
        &mut self,
        entity_id: &EntityId
    ) -> Option<(&mut T1, &mut T2)> {
        let bit1 = self.registry.bit_for::<T1>();
        let bit2 = self.registry.bit_for::<T2>();
        let mask = (1u64 << bit1) | (1u64 << bit2);

        if let Some(&entity_mask) = self.meta.get(entity_id) {
            if (entity_mask & mask) == mask {
                unsafe {
                    let stores_ptr = &mut self.stores as *mut HashMap<
                        TypeId,
                        Box<dyn Any + Send + Sync>
                    >;

                    let store1 = (*stores_ptr)
                        .get_mut(&TypeId::of::<T1>())
                        .unwrap()
                        .downcast_mut::<Store<T1>>()
                        .unwrap();
                    let store2 = (*stores_ptr)
                        .get_mut(&TypeId::of::<T2>())
                        .unwrap()
                        .downcast_mut::<Store<T2>>()
                        .unwrap();

                    let comp1 = store1.get_mut(entity_id)?;
                    let comp2 = store2.get_mut(entity_id)?;

                    return Some((comp1, comp2));
                }
            }
        }
        None
    }

    fn get_component_stores_for_entity3<T1: Component, T2: Component, T3: Component>(
        &mut self,
        entity_id: &EntityId
    ) -> Option<(&mut T1, &mut T2, &mut T3)> {
        let bit1 = self.registry.bit_for::<T1>();
        let bit2 = self.registry.bit_for::<T2>();
        let bit3 = self.registry.bit_for::<T3>();
        let mask = (1u64 << bit1) | (1u64 << bit2) | (1u64 << bit3);

        if let Some(&entity_mask) = self.meta.get(entity_id) {
            if (entity_mask & mask) == mask {
                unsafe {
                    let stores_ptr = &mut self.stores as *mut HashMap<
                        TypeId,
                        Box<dyn Any + Send + Sync>
                    >;

                    let store1 = (*stores_ptr)
                        .get_mut(&TypeId::of::<T1>())
                        .unwrap()
                        .downcast_mut::<Store<T1>>()
                        .unwrap();
                    let store2 = (*stores_ptr)
                        .get_mut(&TypeId::of::<T2>())
                        .unwrap()
                        .downcast_mut::<Store<T2>>()
                        .unwrap();
                    let store3 = (*stores_ptr)
                        .get_mut(&TypeId::of::<T3>())
                        .unwrap()
                        .downcast_mut::<Store<T3>>()
                        .unwrap();

                    let comp1 = store1.get_mut(entity_id)?;
                    let comp2 = store2.get_mut(entity_id)?;
                    let comp3 = store3.get_mut(entity_id)?;

                    return Some((comp1, comp2, comp3));
                }
            }
        }
        None
    }

    fn get_component_stores_for_entity4<T1: Component, T2: Component, T3: Component, T4: Component>(
        &mut self,
        entity_id: &EntityId
    ) -> Option<(&mut T1, &mut T2, &mut T3, &mut T4)> {
        let bit1 = self.registry.bit_for::<T1>();
        let bit2 = self.registry.bit_for::<T2>();
        let bit3 = self.registry.bit_for::<T3>();
        let bit4 = self.registry.bit_for::<T4>();
        let mask = (1u64 << bit1) | (1u64 << bit2) | (1u64 << bit3) | (1u64 << bit4);

        if let Some(&entity_mask) = self.meta.get(entity_id) {
            if (entity_mask & mask) == mask {
                unsafe {
                    let stores_ptr = &mut self.stores as *mut HashMap<TypeId, Box<dyn Any + Send + Sync>>;

                    let store1 = (*stores_ptr).get_mut(&TypeId::of::<T1>()).unwrap().downcast_mut::<Store<T1>>().unwrap();
                    let store2 = (*stores_ptr).get_mut(&TypeId::of::<T2>()).unwrap().downcast_mut::<Store<T2>>().unwrap();
                    let store3 = (*stores_ptr).get_mut(&TypeId::of::<T3>()).unwrap().downcast_mut::<Store<T3>>().unwrap();
                    let store4 = (*stores_ptr).get_mut(&TypeId::of::<T4>()).unwrap().downcast_mut::<Store<T4>>().unwrap();

                    let comp1 = store1.get_mut(entity_id)?;
                    let comp2 = store2.get_mut(entity_id)?;
                    let comp3 = store3.get_mut(entity_id)?;
                    let comp4 = store4.get_mut(entity_id)?;

                    return Some((comp1, comp2, comp3, comp4));
                }
            }
        }
        None
    }

    fn get_component_stores_for_entity5<T1: Component, T2: Component, T3: Component, T4: Component, T5: Component>(
        &mut self,
        entity_id: &EntityId
    ) -> Option<(&mut T1, &mut T2, &mut T3, &mut T4, &mut T5)> {
        let bit1 = self.registry.bit_for::<T1>();
        let bit2 = self.registry.bit_for::<T2>();
        let bit3 = self.registry.bit_for::<T3>();
        let bit4 = self.registry.bit_for::<T4>();
        let bit5 = self.registry.bit_for::<T5>();
        let mask = (1u64 << bit1) | (1u64 << bit2) | (1u64 << bit3) | (1u64 << bit4) | (1u64 << bit5);

        if let Some(&entity_mask) = self.meta.get(entity_id) {
            if (entity_mask & mask) == mask {
                unsafe {
                    let stores_ptr = &mut self.stores as *mut HashMap<TypeId, Box<dyn Any + Send + Sync>>;

                    let store1 = (*stores_ptr).get_mut(&TypeId::of::<T1>()).unwrap().downcast_mut::<Store<T1>>().unwrap();
                    let store2 = (*stores_ptr).get_mut(&TypeId::of::<T2>()).unwrap().downcast_mut::<Store<T2>>().unwrap();
                    let store3 = (*stores_ptr).get_mut(&TypeId::of::<T3>()).unwrap().downcast_mut::<Store<T3>>().unwrap();
                    let store4 = (*stores_ptr).get_mut(&TypeId::of::<T4>()).unwrap().downcast_mut::<Store<T4>>().unwrap();
                    let store5 = (*stores_ptr).get_mut(&TypeId::of::<T5>()).unwrap().downcast_mut::<Store<T5>>().unwrap();

                    let comp1 = store1.get_mut(entity_id)?;
                    let comp2 = store2.get_mut(entity_id)?;
                    let comp3 = store3.get_mut(entity_id)?;
                    let comp4 = store4.get_mut(entity_id)?;
                    let comp5 = store5.get_mut(entity_id)?;

                    return Some((comp1, comp2, comp3, comp4, comp5));
                }
            }
        }
        None
    }

    fn get_component_stores_for_entity6<T1: Component, T2: Component, T3: Component, T4: Component, T5: Component, T6: Component>(
        &mut self,
        entity_id: &EntityId
    ) -> Option<(&mut T1, &mut T2, &mut T3, &mut T4, &mut T5, &mut T6)> {
        let bits = [
            self.registry.bit_for::<T1>(),
            self.registry.bit_for::<T2>(),
            self.registry.bit_for::<T3>(),
            self.registry.bit_for::<T4>(),
            self.registry.bit_for::<T5>(),
            self.registry.bit_for::<T6>(),
        ];
        let mask = bits.iter().fold(0u64, |acc, &bit| acc | (1u64 << bit));

        if let Some(&entity_mask) = self.meta.get(entity_id) {
            if (entity_mask & mask) == mask {
                unsafe {
                    let stores_ptr = &mut self.stores as *mut HashMap<TypeId, Box<dyn Any + Send + Sync>>;
                    let store1 = (*stores_ptr).get_mut(&TypeId::of::<T1>()).unwrap().downcast_mut::<Store<T1>>().unwrap();
                    let store2 = (*stores_ptr).get_mut(&TypeId::of::<T2>()).unwrap().downcast_mut::<Store<T2>>().unwrap();
                    let store3 = (*stores_ptr).get_mut(&TypeId::of::<T3>()).unwrap().downcast_mut::<Store<T3>>().unwrap();
                    let store4 = (*stores_ptr).get_mut(&TypeId::of::<T4>()).unwrap().downcast_mut::<Store<T4>>().unwrap();
                    let store5 = (*stores_ptr).get_mut(&TypeId::of::<T5>()).unwrap().downcast_mut::<Store<T5>>().unwrap();
                    let store6 = (*stores_ptr).get_mut(&TypeId::of::<T6>()).unwrap().downcast_mut::<Store<T6>>().unwrap();

                    return Some((
                        store1.get_mut(entity_id)?,
                        store2.get_mut(entity_id)?,
                        store3.get_mut(entity_id)?,
                        store4.get_mut(entity_id)?,
                        store5.get_mut(entity_id)?,
                        store6.get_mut(entity_id)?,
                    ));
                }
            }
        }
        None
    }

    fn get_component_stores_for_entity7<T1: Component, T2: Component, T3: Component, T4: Component, T5: Component, T6: Component, T7: Component>(
        &mut self,
        entity_id: &EntityId
    ) -> Option<(&mut T1, &mut T2, &mut T3, &mut T4, &mut T5, &mut T6, &mut T7)> {
        let bits = [
            self.registry.bit_for::<T1>(),
            self.registry.bit_for::<T2>(),
            self.registry.bit_for::<T3>(),
            self.registry.bit_for::<T4>(),
            self.registry.bit_for::<T5>(),
            self.registry.bit_for::<T6>(),
            self.registry.bit_for::<T7>(),
        ];
        let mask = bits.iter().fold(0u64, |acc, &bit| acc | (1u64 << bit));

        if let Some(&entity_mask) = self.meta.get(entity_id) {
            if (entity_mask & mask) == mask {
                unsafe {
                    let stores_ptr = &mut self.stores as *mut HashMap<TypeId, Box<dyn Any + Send + Sync>>;
                    let store1 = (*stores_ptr).get_mut(&TypeId::of::<T1>()).unwrap().downcast_mut::<Store<T1>>().unwrap();
                    let store2 = (*stores_ptr).get_mut(&TypeId::of::<T2>()).unwrap().downcast_mut::<Store<T2>>().unwrap();
                    let store3 = (*stores_ptr).get_mut(&TypeId::of::<T3>()).unwrap().downcast_mut::<Store<T3>>().unwrap();
                    let store4 = (*stores_ptr).get_mut(&TypeId::of::<T4>()).unwrap().downcast_mut::<Store<T4>>().unwrap();
                    let store5 = (*stores_ptr).get_mut(&TypeId::of::<T5>()).unwrap().downcast_mut::<Store<T5>>().unwrap();
                    let store6 = (*stores_ptr).get_mut(&TypeId::of::<T6>()).unwrap().downcast_mut::<Store<T6>>().unwrap();
                    let store7 = (*stores_ptr).get_mut(&TypeId::of::<T7>()).unwrap().downcast_mut::<Store<T7>>().unwrap();

                    return Some((
                        store1.get_mut(entity_id)?,
                        store2.get_mut(entity_id)?,
                        store3.get_mut(entity_id)?,
                        store4.get_mut(entity_id)?,
                        store5.get_mut(entity_id)?,
                        store6.get_mut(entity_id)?,
                        store7.get_mut(entity_id)?,
                    ));
                }
            }
        }
        None
    }

    fn get_component_stores_for_entity8<T1: Component, T2: Component, T3: Component, T4: Component, T5: Component, T6: Component, T7: Component, T8: Component>(
        &mut self,
        entity_id: &EntityId
    ) -> Option<(&mut T1, &mut T2, &mut T3, &mut T4, &mut T5, &mut T6, &mut T7, &mut T8)> {
        let bits = [
            self.registry.bit_for::<T1>(),
            self.registry.bit_for::<T2>(),
            self.registry.bit_for::<T3>(),
            self.registry.bit_for::<T4>(),
            self.registry.bit_for::<T5>(),
            self.registry.bit_for::<T6>(),
            self.registry.bit_for::<T7>(),
            self.registry.bit_for::<T8>(),
        ];
        let mask = bits.iter().fold(0u64, |acc, &bit| acc | (1u64 << bit));

        if let Some(&entity_mask) = self.meta.get(entity_id) {
            if (entity_mask & mask) == mask {
                unsafe {
                    let stores_ptr = &mut self.stores as *mut HashMap<TypeId, Box<dyn Any + Send + Sync>>;
                    let store1 = (*stores_ptr).get_mut(&TypeId::of::<T1>()).unwrap().downcast_mut::<Store<T1>>().unwrap();
                    let store2 = (*stores_ptr).get_mut(&TypeId::of::<T2>()).unwrap().downcast_mut::<Store<T2>>().unwrap();
                    let store3 = (*stores_ptr).get_mut(&TypeId::of::<T3>()).unwrap().downcast_mut::<Store<T3>>().unwrap();
                    let store4 = (*stores_ptr).get_mut(&TypeId::of::<T4>()).unwrap().downcast_mut::<Store<T4>>().unwrap();
                    let store5 = (*stores_ptr).get_mut(&TypeId::of::<T5>()).unwrap().downcast_mut::<Store<T5>>().unwrap();
                    let store6 = (*stores_ptr).get_mut(&TypeId::of::<T6>()).unwrap().downcast_mut::<Store<T6>>().unwrap();
                    let store7 = (*stores_ptr).get_mut(&TypeId::of::<T7>()).unwrap().downcast_mut::<Store<T7>>().unwrap();
                    let store8 = (*stores_ptr).get_mut(&TypeId::of::<T8>()).unwrap().downcast_mut::<Store<T8>>().unwrap();

                    return Some((
                        store1.get_mut(entity_id)?,
                        store2.get_mut(entity_id)?,
                        store3.get_mut(entity_id)?,
                        store4.get_mut(entity_id)?,
                        store5.get_mut(entity_id)?,
                        store6.get_mut(entity_id)?,
                        store7.get_mut(entity_id)?,
                        store8.get_mut(entity_id)?,
                    ));
                }
            }
        }
        None
    }

    fn get_component_stores_for_entity9<T1: Component, T2: Component, T3: Component, T4: Component, T5: Component, T6: Component, T7: Component, T8: Component, T9: Component>(
        &mut self,
        entity_id: &EntityId
    ) -> Option<(&mut T1, &mut T2, &mut T3, &mut T4, &mut T5, &mut T6, &mut T7, &mut T8, &mut T9)> {
        let bits = [
            self.registry.bit_for::<T1>(),
            self.registry.bit_for::<T2>(),
            self.registry.bit_for::<T3>(),
            self.registry.bit_for::<T4>(),
            self.registry.bit_for::<T5>(),
            self.registry.bit_for::<T6>(),
            self.registry.bit_for::<T7>(),
            self.registry.bit_for::<T8>(),
            self.registry.bit_for::<T9>(),
        ];
        let mask = bits.iter().fold(0u64, |acc, &bit| acc | (1u64 << bit));

        if let Some(&entity_mask) = self.meta.get(entity_id) {
            if (entity_mask & mask) == mask {
                unsafe {
                    let stores_ptr = &mut self.stores as *mut HashMap<TypeId, Box<dyn Any + Send + Sync>>;
                    let store1 = (*stores_ptr).get_mut(&TypeId::of::<T1>()).unwrap().downcast_mut::<Store<T1>>().unwrap();
                    let store2 = (*stores_ptr).get_mut(&TypeId::of::<T2>()).unwrap().downcast_mut::<Store<T2>>().unwrap();
                    let store3 = (*stores_ptr).get_mut(&TypeId::of::<T3>()).unwrap().downcast_mut::<Store<T3>>().unwrap();
                    let store4 = (*stores_ptr).get_mut(&TypeId::of::<T4>()).unwrap().downcast_mut::<Store<T4>>().unwrap();
                    let store5 = (*stores_ptr).get_mut(&TypeId::of::<T5>()).unwrap().downcast_mut::<Store<T5>>().unwrap();
                    let store6 = (*stores_ptr).get_mut(&TypeId::of::<T6>()).unwrap().downcast_mut::<Store<T6>>().unwrap();
                    let store7 = (*stores_ptr).get_mut(&TypeId::of::<T7>()).unwrap().downcast_mut::<Store<T7>>().unwrap();
                    let store8 = (*stores_ptr).get_mut(&TypeId::of::<T8>()).unwrap().downcast_mut::<Store<T8>>().unwrap();
                    let store9 = (*stores_ptr).get_mut(&TypeId::of::<T9>()).unwrap().downcast_mut::<Store<T9>>().unwrap();

                    return Some((
                        store1.get_mut(entity_id)?,
                        store2.get_mut(entity_id)?,
                        store3.get_mut(entity_id)?,
                        store4.get_mut(entity_id)?,
                        store5.get_mut(entity_id)?,
                        store6.get_mut(entity_id)?,
                        store7.get_mut(entity_id)?,
                        store8.get_mut(entity_id)?,
                        store9.get_mut(entity_id)?,
                    ));
                }
            }
        }
        None
    }

    fn get_component_stores_for_entity10<T1: Component, T2: Component, T3: Component, T4: Component, T5: Component, T6: Component, T7: Component, T8: Component, T9: Component, T10: Component>(
        &mut self,
        entity_id: &EntityId
    ) -> Option<(&mut T1, &mut T2, &mut T3, &mut T4, &mut T5, &mut T6, &mut T7, &mut T8, &mut T9, &mut T10)> {
        let bits = [
            self.registry.bit_for::<T1>(),
            self.registry.bit_for::<T2>(),
            self.registry.bit_for::<T3>(),
            self.registry.bit_for::<T4>(),
            self.registry.bit_for::<T5>(),
            self.registry.bit_for::<T6>(),
            self.registry.bit_for::<T7>(),
            self.registry.bit_for::<T8>(),
            self.registry.bit_for::<T9>(),
            self.registry.bit_for::<T10>(),
        ];
        let mask = bits.iter().fold(0u64, |acc, &bit| acc | (1u64 << bit));

        if let Some(&entity_mask) = self.meta.get(entity_id) {
            if (entity_mask & mask) == mask {
                unsafe {
                    let stores_ptr = &mut self.stores as *mut HashMap<TypeId, Box<dyn Any + Send + Sync>>;
                    let store1 = (*stores_ptr).get_mut(&TypeId::of::<T1>()).unwrap().downcast_mut::<Store<T1>>().unwrap();
                    let store2 = (*stores_ptr).get_mut(&TypeId::of::<T2>()).unwrap().downcast_mut::<Store<T2>>().unwrap();
                    let store3 = (*stores_ptr).get_mut(&TypeId::of::<T3>()).unwrap().downcast_mut::<Store<T3>>().unwrap();
                    let store4 = (*stores_ptr).get_mut(&TypeId::of::<T4>()).unwrap().downcast_mut::<Store<T4>>().unwrap();
                    let store5 = (*stores_ptr).get_mut(&TypeId::of::<T5>()).unwrap().downcast_mut::<Store<T5>>().unwrap();
                    let store6 = (*stores_ptr).get_mut(&TypeId::of::<T6>()).unwrap().downcast_mut::<Store<T6>>().unwrap();
                    let store7 = (*stores_ptr).get_mut(&TypeId::of::<T7>()).unwrap().downcast_mut::<Store<T7>>().unwrap();
                    let store8 = (*stores_ptr).get_mut(&TypeId::of::<T8>()).unwrap().downcast_mut::<Store<T8>>().unwrap();
                    let store9 = (*stores_ptr).get_mut(&TypeId::of::<T9>()).unwrap().downcast_mut::<Store<T9>>().unwrap();
                    let store10 = (*stores_ptr).get_mut(&TypeId::of::<T10>()).unwrap().downcast_mut::<Store<T10>>().unwrap();

                    return Some((
                        store1.get_mut(entity_id)?,
                        store2.get_mut(entity_id)?,
                        store3.get_mut(entity_id)?,
                        store4.get_mut(entity_id)?,
                        store5.get_mut(entity_id)?,
                        store6.get_mut(entity_id)?,
                        store7.get_mut(entity_id)?,
                        store8.get_mut(entity_id)?,
                        store9.get_mut(entity_id)?,
                        store10.get_mut(entity_id)?,
                    ));
                }
            }
        }
        None
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
        use std::boxed::Box; use crate::index::entity_component_system::Insertable;
        let mut v: Vec<Box<dyn crate::index::entity_component_system::Insertable>> = Vec::new();
        $( v.push(Box::new($comp)); )+
        crate::index::entity_component_system::world().insert_dyn(&$entity, v);
        }
    };
}

// Simple macro implementation without paste crate

impl World {
    pub fn query2<F, C1, C2>(&mut self, mut f: F)
        where C1: Component, C2: Component, F: FnMut(&EntityId, &mut C1, &mut C2)
    {
        let bit1 = self.registry.bit_for::<C1>();
        let bit2 = self.registry.bit_for::<C2>();
        let mask = (1u64 << bit1) | (1u64 << bit2);

        let entities: Vec<EntityId> = self.meta
            .iter()
            .filter(|(_, &entity_mask)| (entity_mask & mask) == mask)
            .map(|(id, _)| id.clone())
            .collect();

        for entity_id in entities {
            if let Some((c1, c2)) = self.get_component_stores_for_entity::<C1, C2>(&entity_id) {
                f(&entity_id, c1, c2);
            }
        }
    }

    pub fn query_by_id2<F, C1, C2>(&mut self, id: &EntityId, mut f: F)
        where C1: Component, C2: Component, F: FnMut(&mut C1, &mut C2)
    {
        if let Some((c1, c2)) = self.get_component_stores_for_entity::<C1, C2>(id) {
            f(c1, c2);
        }
    }

    pub fn query3<F, C1, C2, C3>(&mut self, mut f: F)
        where
            C1: Component,
            C2: Component,
            C3: Component,
            F: FnMut(&EntityId, &mut C1, &mut C2, &mut C3)
    {
        let bit1 = self.registry.bit_for::<C1>();
        let bit2 = self.registry.bit_for::<C2>();
        let bit3 = self.registry.bit_for::<C3>();
        let mask = (1u64 << bit1) | (1u64 << bit2) | (1u64 << bit3);

        let entities: Vec<EntityId> = self.meta
            .iter()
            .filter(|(_, &entity_mask)| (entity_mask & mask) == mask)
            .map(|(id, _)| id.clone())
            .collect();

        for entity_id in entities {
            if
                let Some((c1, c2, c3)) = self.get_component_stores_for_entity3::<C1, C2, C3>(
                    &entity_id
                )
            {
                f(&entity_id, c1, c2, c3);
            }
        }
    }

    pub fn query_by_id3<F, C1, C2, C3>(&mut self, id: &EntityId, mut f: F)
        where C1: Component, C2: Component, C3: Component, F: FnMut(&mut C1, &mut C2, &mut C3)
    {
        if let Some((c1, c2, c3)) = self.get_component_stores_for_entity3::<C1, C2, C3>(id) {
            f(c1, c2, c3);
        }
    }

    pub fn query4<F, C1, C2, C3, C4>(&mut self, mut f: F)
        where C1: Component, C2: Component, C3: Component, C4: Component, F: FnMut(&EntityId, &mut C1, &mut C2, &mut C3, &mut C4)
    {
        let bits = [
            self.registry.bit_for::<C1>(),
            self.registry.bit_for::<C2>(),
            self.registry.bit_for::<C3>(),
            self.registry.bit_for::<C4>(),
        ];
        let mask = bits.iter().fold(0u64, |acc, &bit| acc | (1u64 << bit));

        let entities: Vec<EntityId> = self.meta
            .iter()
            .filter(|(_, &entity_mask)| (entity_mask & mask) == mask)
            .map(|(id, _)| id.clone())
            .collect();

        for entity_id in entities {
            if let Some((c1, c2, c3, c4)) = self.get_component_stores_for_entity4::<C1, C2, C3, C4>(&entity_id) {
                f(&entity_id, c1, c2, c3, c4);
            }
        }
    }

    pub fn query_by_id4<F, C1, C2, C3, C4>(&mut self, id: &EntityId, mut f: F)
        where C1: Component, C2: Component, C3: Component, C4: Component, F: FnMut(&mut C1, &mut C2, &mut C3, &mut C4)
    {
        if let Some((c1, c2, c3, c4)) = self.get_component_stores_for_entity4::<C1, C2, C3, C4>(id) {
            f(c1, c2, c3, c4);
        }
    }

    pub fn query5<F, C1, C2, C3, C4, C5>(&mut self, mut f: F)
        where C1: Component, C2: Component, C3: Component, C4: Component, C5: Component, F: FnMut(&EntityId, &mut C1, &mut C2, &mut C3, &mut C4, &mut C5)
    {
        let bits = [
            self.registry.bit_for::<C1>(),
            self.registry.bit_for::<C2>(),
            self.registry.bit_for::<C3>(),
            self.registry.bit_for::<C4>(),
            self.registry.bit_for::<C5>(),
        ];
        let mask = bits.iter().fold(0u64, |acc, &bit| acc | (1u64 << bit));

        let entities: Vec<EntityId> = self.meta
            .iter()
            .filter(|(_, &entity_mask)| (entity_mask & mask) == mask)
            .map(|(id, _)| id.clone())
            .collect();

        for entity_id in entities {
            if let Some((c1, c2, c3, c4, c5)) = self.get_component_stores_for_entity5::<C1, C2, C3, C4, C5>(&entity_id) {
                f(&entity_id, c1, c2, c3, c4, c5);
            }
        }
    }

    pub fn query_by_id5<F, C1, C2, C3, C4, C5>(&mut self, id: &EntityId, mut f: F)
        where C1: Component, C2: Component, C3: Component, C4: Component, C5: Component, F: FnMut(&mut C1, &mut C2, &mut C3, &mut C4, &mut C5)
    {
        if let Some((c1, c2, c3, c4, c5)) = self.get_component_stores_for_entity5::<C1, C2, C3, C4, C5>(id) {
            f(c1, c2, c3, c4, c5);
        }
    }

    pub fn query6<F, C1, C2, C3, C4, C5, C6>(&mut self, mut f: F)
        where C1: Component, C2: Component, C3: Component, C4: Component, C5: Component, C6: Component, F: FnMut(&EntityId, &mut C1, &mut C2, &mut C3, &mut C4, &mut C5, &mut C6)
    {
        let bits = [
            self.registry.bit_for::<C1>(),
            self.registry.bit_for::<C2>(),
            self.registry.bit_for::<C3>(),
            self.registry.bit_for::<C4>(),
            self.registry.bit_for::<C5>(),
            self.registry.bit_for::<C6>(),
        ];
        let mask = bits.iter().fold(0u64, |acc, &bit| acc | (1u64 << bit));

        let entities: Vec<EntityId> = self.meta
            .iter()
            .filter(|(_, &entity_mask)| (entity_mask & mask) == mask)
            .map(|(id, _)| id.clone())
            .collect();

        for entity_id in entities {
            if let Some((c1, c2, c3, c4, c5, c6)) = self.get_component_stores_for_entity6::<C1, C2, C3, C4, C5, C6>(&entity_id) {
                f(&entity_id, c1, c2, c3, c4, c5, c6);
            }
        }
    }

    pub fn query_by_id6<F, C1, C2, C3, C4, C5, C6>(&mut self, id: &EntityId, mut f: F)
        where C1: Component, C2: Component, C3: Component, C4: Component, C5: Component, C6: Component, F: FnMut(&mut C1, &mut C2, &mut C3, &mut C4, &mut C5, &mut C6)
    {
        if let Some((c1, c2, c3, c4, c5, c6)) = self.get_component_stores_for_entity6::<C1, C2, C3, C4, C5, C6>(id) {
            f(c1, c2, c3, c4, c5, c6);
        }
    }

    pub fn query7<F, C1, C2, C3, C4, C5, C6, C7>(&mut self, mut f: F)
        where C1: Component, C2: Component, C3: Component, C4: Component, C5: Component, C6: Component, C7: Component, F: FnMut(&EntityId, &mut C1, &mut C2, &mut C3, &mut C4, &mut C5, &mut C6, &mut C7)
    {
        let bits = [
            self.registry.bit_for::<C1>(),
            self.registry.bit_for::<C2>(),
            self.registry.bit_for::<C3>(),
            self.registry.bit_for::<C4>(),
            self.registry.bit_for::<C5>(),
            self.registry.bit_for::<C6>(),
            self.registry.bit_for::<C7>(),
        ];
        let mask = bits.iter().fold(0u64, |acc, &bit| acc | (1u64 << bit));

        let entities: Vec<EntityId> = self.meta
            .iter()
            .filter(|(_, &entity_mask)| (entity_mask & mask) == mask)
            .map(|(id, _)| id.clone())
            .collect();

        for entity_id in entities {
            if let Some((c1, c2, c3, c4, c5, c6, c7)) = self.get_component_stores_for_entity7::<C1, C2, C3, C4, C5, C6, C7>(&entity_id) {
                f(&entity_id, c1, c2, c3, c4, c5, c6, c7);
            }
        }
    }

    pub fn query_by_id7<F, C1, C2, C3, C4, C5, C6, C7>(&mut self, id: &EntityId, mut f: F)
        where C1: Component, C2: Component, C3: Component, C4: Component, C5: Component, C6: Component, C7: Component, F: FnMut(&mut C1, &mut C2, &mut C3, &mut C4, &mut C5, &mut C6, &mut C7)
    {
        if let Some((c1, c2, c3, c4, c5, c6, c7)) = self.get_component_stores_for_entity7::<C1, C2, C3, C4, C5, C6, C7>(id) {
            f(c1, c2, c3, c4, c5, c6, c7);
        }
    }

    pub fn query8<F, C1, C2, C3, C4, C5, C6, C7, C8>(&mut self, mut f: F)
        where C1: Component, C2: Component, C3: Component, C4: Component, C5: Component, C6: Component, C7: Component, C8: Component, F: FnMut(&EntityId, &mut C1, &mut C2, &mut C3, &mut C4, &mut C5, &mut C6, &mut C7, &mut C8)
    {
        let bits = [
            self.registry.bit_for::<C1>(),
            self.registry.bit_for::<C2>(),
            self.registry.bit_for::<C3>(),
            self.registry.bit_for::<C4>(),
            self.registry.bit_for::<C5>(),
            self.registry.bit_for::<C6>(),
            self.registry.bit_for::<C7>(),
            self.registry.bit_for::<C8>(),
        ];
        let mask = bits.iter().fold(0u64, |acc, &bit| acc | (1u64 << bit));

        let entities: Vec<EntityId> = self.meta
            .iter()
            .filter(|(_, &entity_mask)| (entity_mask & mask) == mask)
            .map(|(id, _)| id.clone())
            .collect();

        for entity_id in entities {
            if let Some((c1, c2, c3, c4, c5, c6, c7, c8)) = self.get_component_stores_for_entity8::<C1, C2, C3, C4, C5, C6, C7, C8>(&entity_id) {
                f(&entity_id, c1, c2, c3, c4, c5, c6, c7, c8);
            }
        }
    }

    pub fn query_by_id8<F, C1, C2, C3, C4, C5, C6, C7, C8>(&mut self, id: &EntityId, mut f: F)
        where C1: Component, C2: Component, C3: Component, C4: Component, C5: Component, C6: Component, C7: Component, C8: Component, F: FnMut(&mut C1, &mut C2, &mut C3, &mut C4, &mut C5, &mut C6, &mut C7, &mut C8)
    {
        if let Some((c1, c2, c3, c4, c5, c6, c7, c8)) = self.get_component_stores_for_entity8::<C1, C2, C3, C4, C5, C6, C7, C8>(id) {
            f(c1, c2, c3, c4, c5, c6, c7, c8);
        }
    }

    pub fn query9<F, C1, C2, C3, C4, C5, C6, C7, C8, C9>(&mut self, mut f: F)
        where C1: Component, C2: Component, C3: Component, C4: Component, C5: Component, C6: Component, C7: Component, C8: Component, C9: Component, F: FnMut(&EntityId, &mut C1, &mut C2, &mut C3, &mut C4, &mut C5, &mut C6, &mut C7, &mut C8, &mut C9)
    {
        let bits = [
            self.registry.bit_for::<C1>(),
            self.registry.bit_for::<C2>(),
            self.registry.bit_for::<C3>(),
            self.registry.bit_for::<C4>(),
            self.registry.bit_for::<C5>(),
            self.registry.bit_for::<C6>(),
            self.registry.bit_for::<C7>(),
            self.registry.bit_for::<C8>(),
            self.registry.bit_for::<C9>(),
        ];
        let mask = bits.iter().fold(0u64, |acc, &bit| acc | (1u64 << bit));

        let entities: Vec<EntityId> = self.meta
            .iter()
            .filter(|(_, &entity_mask)| (entity_mask & mask) == mask)
            .map(|(id, _)| id.clone())
            .collect();

        for entity_id in entities {
            if let Some((c1, c2, c3, c4, c5, c6, c7, c8, c9)) = self.get_component_stores_for_entity9::<C1, C2, C3, C4, C5, C6, C7, C8, C9>(&entity_id) {
                f(&entity_id, c1, c2, c3, c4, c5, c6, c7, c8, c9);
            }
        }
    }

    pub fn query_by_id9<F, C1, C2, C3, C4, C5, C6, C7, C8, C9>(&mut self, id: &EntityId, mut f: F)
        where C1: Component, C2: Component, C3: Component, C4: Component, C5: Component, C6: Component, C7: Component, C8: Component, C9: Component, F: FnMut(&mut C1, &mut C2, &mut C3, &mut C4, &mut C5, &mut C6, &mut C7, &mut C8, &mut C9)
    {
        if let Some((c1, c2, c3, c4, c5, c6, c7, c8, c9)) = self.get_component_stores_for_entity9::<C1, C2, C3, C4, C5, C6, C7, C8, C9>(id) {
            f(c1, c2, c3, c4, c5, c6, c7, c8, c9);
        }
    }

    pub fn query10<F, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10>(&mut self, mut f: F)
        where C1: Component, C2: Component, C3: Component, C4: Component, C5: Component, C6: Component, C7: Component, C8: Component, C9: Component, C10: Component, F: FnMut(&EntityId, &mut C1, &mut C2, &mut C3, &mut C4, &mut C5, &mut C6, &mut C7, &mut C8, &mut C9, &mut C10)
    {
        let bits = [
            self.registry.bit_for::<C1>(),
            self.registry.bit_for::<C2>(),
            self.registry.bit_for::<C3>(),
            self.registry.bit_for::<C4>(),
            self.registry.bit_for::<C5>(),
            self.registry.bit_for::<C6>(),
            self.registry.bit_for::<C7>(),
            self.registry.bit_for::<C8>(),
            self.registry.bit_for::<C9>(),
            self.registry.bit_for::<C10>(),
        ];
        let mask = bits.iter().fold(0u64, |acc, &bit| acc | (1u64 << bit));

        let entities: Vec<EntityId> = self.meta
            .iter()
            .filter(|(_, &entity_mask)| (entity_mask & mask) == mask)
            .map(|(id, _)| id.clone())
            .collect();

        for entity_id in entities {
            if let Some((c1, c2, c3, c4, c5, c6, c7, c8, c9, c10)) = self.get_component_stores_for_entity10::<C1, C2, C3, C4, C5, C6, C7, C8, C9, C10>(&entity_id) {
                f(&entity_id, c1, c2, c3, c4, c5, c6, c7, c8, c9, c10);
            }
        }
    }

    pub fn query_by_id10<F, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10>(&mut self, id: &EntityId, mut f: F)
        where C1: Component, C2: Component, C3: Component, C4: Component, C5: Component, C6: Component, C7: Component, C8: Component, C9: Component, C10: Component, F: FnMut(&mut C1, &mut C2, &mut C3, &mut C4, &mut C5, &mut C6, &mut C7, &mut C8, &mut C9, &mut C10)
    {
        if let Some((c1, c2, c3, c4, c5, c6, c7, c8, c9, c10)) = self.get_component_stores_for_entity10::<C1, C2, C3, C4, C5, C6, C7, C8, C9, C10>(id) {
            f(c1, c2, c3, c4, c5, c6, c7, c8, c9, c10);
        }
    }
}

// convenience front‑end macros --------------------------------------------

#[macro_export]
macro_rules! query {
    (($c1:ty, $c2:ty, $c3:ty, $c4:ty, $c5:ty, $c6:ty, $c7:ty, $c8:ty, $c9:ty, $c10:ty), | $id:ident, $a1:ident, $a2:ident, $a3:ident, $a4:ident, $a5:ident, $a6:ident, $a7:ident, $a8:ident, $a9:ident, $a10:ident | $body:block) => {
        crate::index::entity_component_system::world().query10::<_, $c1, $c2, $c3, $c4, $c5, $c6, $c7, $c8, $c9, $c10>(|$id, $a1, $a2, $a3, $a4, $a5, $a6, $a7, $a8, $a9, $a10| $body)
    };
    (($c1:ty, $c2:ty, $c3:ty, $c4:ty, $c5:ty, $c6:ty, $c7:ty, $c8:ty, $c9:ty), | $id:ident, $a1:ident, $a2:ident, $a3:ident, $a4:ident, $a5:ident, $a6:ident, $a7:ident, $a8:ident, $a9:ident | $body:block) => {
        crate::index::entity_component_system::world().query9::<_, $c1, $c2, $c3, $c4, $c5, $c6, $c7, $c8, $c9>(|$id, $a1, $a2, $a3, $a4, $a5, $a6, $a7, $a8, $a9| $body)
    };
    (($c1:ty, $c2:ty, $c3:ty, $c4:ty, $c5:ty, $c6:ty, $c7:ty, $c8:ty), | $id:ident, $a1:ident, $a2:ident, $a3:ident, $a4:ident, $a5:ident, $a6:ident, $a7:ident, $a8:ident | $body:block) => {
        crate::index::entity_component_system::world().query8::<_, $c1, $c2, $c3, $c4, $c5, $c6, $c7, $c8>(|$id, $a1, $a2, $a3, $a4, $a5, $a6, $a7, $a8| $body)
    };
    (($c1:ty, $c2:ty, $c3:ty, $c4:ty, $c5:ty, $c6:ty, $c7:ty), | $id:ident, $a1:ident, $a2:ident, $a3:ident, $a4:ident, $a5:ident, $a6:ident, $a7:ident | $body:block) => {
        crate::index::entity_component_system::world().query7::<_, $c1, $c2, $c3, $c4, $c5, $c6, $c7>(|$id, $a1, $a2, $a3, $a4, $a5, $a6, $a7| $body)
    };
    (($c1:ty, $c2:ty, $c3:ty, $c4:ty, $c5:ty, $c6:ty), | $id:ident, $a1:ident, $a2:ident, $a3:ident, $a4:ident, $a5:ident, $a6:ident | $body:block) => {
        crate::index::entity_component_system::world().query6::<_, $c1, $c2, $c3, $c4, $c5, $c6>(|$id, $a1, $a2, $a3, $a4, $a5, $a6| $body)
    };
    (($c1:ty, $c2:ty, $c3:ty, $c4:ty, $c5:ty), | $id:ident, $a1:ident, $a2:ident, $a3:ident, $a4:ident, $a5:ident | $body:block) => {
        crate::index::entity_component_system::world().query5::<_, $c1, $c2, $c3, $c4, $c5>(|$id, $a1, $a2, $a3, $a4, $a5| $body)
    };
    (($c1:ty, $c2:ty, $c3:ty, $c4:ty), | $id:ident, $a1:ident, $a2:ident, $a3:ident, $a4:ident | $body:block) => {
        crate::index::entity_component_system::world().query4::<_, $c1, $c2, $c3, $c4>(|$id, $a1, $a2, $a3, $a4| $body)
    };
    (($c1:ty, $c2:ty, $c3:ty), | $id:ident, $a1:ident, $a2:ident, $a3:ident | $body:block) => {
        crate::index::entity_component_system::world().query3::<_, $c1, $c2, $c3>(|$id, $a1, $a2, $a3| $body)
    };
    (($c1:ty, $c2:ty), | $id:ident, $a1:ident, $a2:ident | $body:block) => {
        crate::index::entity_component_system::world().query2::<_, $c1, $c2>(|$id, $a1, $a2| $body)
    };
}


#[macro_export]
macro_rules! query_by_id {
    ($eid:expr, ($c1:ty, $c2:ty, $c3:ty, $c4:ty, $c5:ty, $c6:ty, $c7:ty, $c8:ty, $c9:ty, $c10:ty), | $a1:ident, $a2:ident, $a3:ident, $a4:ident, $a5:ident, $a6:ident, $a7:ident, $a8:ident, $a9:ident, $a10:ident | $body:block) => {
        $crate::world().query_by_id10::<_, $c1, $c2, $c3, $c4, $c5, $c6, $c7, $c8, $c9, $c10>(&$eid, |$a1, $a2, $a3, $a4, $a5, $a6, $a7, $a8, $a9, $a10| $body)
    };
    ($eid:expr, ($c1:ty, $c2:ty, $c3:ty, $c4:ty, $c5:ty, $c6:ty, $c7:ty, $c8:ty, $c9:ty), | $a1:ident, $a2:ident, $a3:ident, $a4:ident, $a5:ident, $a6:ident, $a7:ident, $a8:ident, $a9:ident | $body:block) => {
        $crate::world().query_by_id9::<_, $c1, $c2, $c3, $c4, $c5, $c6, $c7, $c8, $c9>(&$eid, |$a1, $a2, $a3, $a4, $a5, $a6, $a7, $a8, $a9| $body)
    };
    ($eid:expr, ($c1:ty, $c2:ty, $c3:ty, $c4:ty, $c5:ty, $c6:ty, $c7:ty, $c8:ty), | $a1:ident, $a2:ident, $a3:ident, $a4:ident, $a5:ident, $a6:ident, $a7:ident, $a8:ident | $body:block) => {
        $crate::world().query_by_id8::<_, $c1, $c2, $c3, $c4, $c5, $c6, $c7, $c8>(&$eid, |$a1, $a2, $a3, $a4, $a5, $a6, $a7, $a8| $body)
    };
    ($eid:expr, ($c1:ty, $c2:ty, $c3:ty, $c4:ty, $c5:ty, $c6:ty, $c7:ty), | $a1:ident, $a2:ident, $a3:ident, $a4:ident, $a5:ident, $a6:ident, $a7:ident | $body:block) => {
        $crate::world().query_by_id7::<_, $c1, $c2, $c3, $c4, $c5, $c6, $c7>(&$eid, |$a1, $a2, $a3, $a4, $a5, $a6, $a7| $body)
    };
    ($eid:expr, ($c1:ty, $c2:ty, $c3:ty, $c4:ty, $c5:ty, $c6:ty), | $a1:ident, $a2:ident, $a3:ident, $a4:ident, $a5:ident, $a6:ident | $body:block) => {
        $crate::world().query_by_id6::<_, $c1, $c2, $c3, $c4, $c5, $c6>(&$eid, |$a1, $a2, $a3, $a4, $a5, $a6| $body)
    };
    ($eid:expr, ($c1:ty, $c2:ty, $c3:ty, $c4:ty, $c5:ty), | $a1:ident, $a2:ident, $a3:ident, $a4:ident, $a5:ident | $body:block) => {
        $crate::world().query_by_id5::<_, $c1, $c2, $c3, $c4, $c5>(&$eid, |$a1, $a2, $a3, $a4, $a5| $body)
    };
    ($eid:expr, ($c1:ty, $c2:ty, $c3:ty, $c4:ty), | $a1:ident, $a2:ident, $a3:ident, $a4:ident | $body:block) => {
        $crate::world().query_by_id4::<_, $c1, $c2, $c3, $c4>(&$eid, |$a1, $a2, $a3, $a4| $body)
    };
    ($eid:expr, ($c1:ty, $c2:ty, $c3:ty), | $a1:ident, $a2:ident, $a3:ident | $body:block) => {
        $crate::world().query_by_id3::<_, $c1, $c2, $c3>(&$eid, |$a1, $a2, $a3| $body)
    };
    ($eid:expr, ($c1:ty, $c2:ty), | $a1:ident, $a2:ident | $body:block) => {
        $crate::world().query_by_id2::<_, $c1, $c2>(&$eid, |$a1, $a2| $body)
    };
}
