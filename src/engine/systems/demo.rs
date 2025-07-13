// Import only the ECS functionality from entityComponentSystem
mod engine {
    pub mod systems {
        #[path = "../../engine/systems/entityComponentSystem.rs"]
        pub mod entity_component_system;
        pub use entity_component_system::*;
    }
}

use engine::systems::*;

pub static PLAYER_ENTITY_ID: Lazy<RwLock<Option<EntityId>>> = Lazy::new(|| RwLock::new(None));

#[derive(Debug)]
pub struct Transform(pub f32, pub f32);
#[derive(Debug)]
pub struct Velocity(pub f32, pub f32);
#[derive(Debug)]
pub struct Health(pub u32);
#[derive(Debug)]
pub struct Collider(pub u32);
#[derive(Debug)]
pub struct Armor(pub u32);
#[derive(Debug)]
pub struct Weapon(pub String);

fn main() {
    let player = spawn();
    *PLAYER_ENTITY_ID.write().unwrap() = Some(player.clone());
    insert_many!(
        player,
        Transform(0.0, 0.0),
        Velocity(1.0, 0.0),
        Health(100),
        Collider(0),
        Armor(50),
        Weapon("Sword".to_string())
    );

    // Test 2-component query
    query!((Transform, Velocity), |id, t, v| {
        println!("2-comp {id}: {:?} {:?}", t, v);
    });

    // Test 4-component query
    query!((Transform, Velocity, Health, Collider), |id, t, v, h, c| {
        println!("4-comp {id}: {:?} {:?} {:?} {:?}", t, v, h, c);
    });

    // Test 6-component query
    query!((Transform, Velocity, Health, Collider, Armor, Weapon), |id, t, v, h, c, a, w| {
        println!("6-comp {id}: {:?} {:?} {:?} {:?} {:?} {:?}", t, v, h, c, a, w);
    });

    if let Some(pid) = PLAYER_ENTITY_ID.read().unwrap().clone() {
        query_by_id!(pid, (Transform, Health), |t, h| {
            h.0 -= 10;
            println!("player now {:?}, hp {:?}", t, h);
        });
    }
}
