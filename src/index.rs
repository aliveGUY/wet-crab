use std::sync::{ Arc, RwLock };

use glow::HasContext;
use once_cell::sync::Lazy;

#[path = "engine/mod.rs"]
#[macro_use]
pub mod engine;
#[path = "game/mod.rs"]
pub mod game;

use engine::*;
use game::*;

use crate::index::game::physics_system::PhysicsSystem;

pub static PLAYER_ENTITY_ID: Lazy<RwLock<Option<EntityId>>> = Lazy::new(|| RwLock::new(None));

fn spawn_wireframe_shapes(_gl: &glow::Context) {
    use crate::index::engine::components::{ Transform, Shape };

    // Sphere at (-3, 0, 0)
    let sphere_entity = spawn();
    let sphere_shape = Shape::Sphere { radius: 1.0 };
    insert_many!(sphere_entity, Transform::new(-3.0, 0.0, 0.0), sphere_shape);

    // Capsule at (-1, 0, 0)
    let capsule_entity = spawn();
    let capsule_shape = Shape::Capsule { radius: 0.5, height: 2.0 };
    insert_many!(capsule_entity, Transform::new(-1.0, 0.0, 0.0), capsule_shape);

    // Box at (1, 0, 0)
    let box_entity = spawn();
    let box_shape = Shape::Box { half_extents: [0.5, 0.5, 0.5] };
    insert_many!(box_entity, Transform::new(1.0, 0.0, 0.0), box_shape);

    // Cylinder at (3, 0, 0)
    let cylinder_entity = spawn();
    let cylinder_shape = Shape::Cylinder { radius: 0.8, height: 1.5 };
    insert_many!(cylinder_entity, Transform::new(3.0, 0.0, 0.0), cylinder_shape);

    println!("✅ Spawned wireframe shapes: Sphere, Capsule, Box, Cylinder");
}

pub struct Program {
    gl: glow::Context,
}

impl Program {
    pub fn new(gl: glow::Context) -> Result<Self, String> {
        initialize_asset_manager(&gl);

        EventSystem::subscribe(EventType::Move, Arc::new(MovementSystem));
        EventSystem::subscribe(EventType::RotateCamera, Arc::new(CameraRotationSystem));

        InterfaceSystem::update_entity_tree_global();

        load_world!("src/assets/scenes/test_world.json");

        // Spawn wireframe shapes at hardcoded coordinates
        spawn_wireframe_shapes(&gl);

        // Update UI to reflect loaded entities
        InterfaceSystem::update_entity_tree_global();

        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LESS);
            gl.depth_mask(true);

            gl.enable(glow::CULL_FACE);
            gl.cull_face(glow::BACK);
            gl.front_face(glow::CCW);

            let depth_bits = gl.get_parameter_i32(glow::DEPTH_BITS);
            if depth_bits == 0 {
                eprintln!("[WARNING] No depth buffer detected in Program::new()");
                eprintln!("[WARNING] Depth testing may not work correctly");
            } else {
                println!("[DEBUG] Program initialized with {} bit depth buffer", depth_bits);
            }
        }

        println!("✅ Program initialized successfully with ECS-based architecture");

        Ok(Self { gl })
    }

    pub fn render(&mut self, width: u32, height: u32, delta_time: f32) {
        let mut viewport = [0i32; 4];
        let mut program = 0i32;
        let mut depth_func = 0;
        let mut writemask = 0i32;
        unsafe {
            self.gl.get_parameter_i32_slice(glow::VIEWPORT, &mut viewport);
            self.gl.get_parameter_i32_slice(
                glow::CURRENT_PROGRAM,
                std::slice::from_mut(&mut program)
            );
            self.gl.get_parameter_i32_slice(
                glow::DEPTH_FUNC,
                std::slice::from_mut(&mut depth_func)
            );
            self.gl.get_parameter_i32_slice(
                glow::DEPTH_WRITEMASK,
                std::slice::from_mut(&mut writemask)
            );
            self.gl.enable(glow::DEPTH_TEST);
            self.gl.depth_func(glow::LESS);
            self.gl.depth_mask(true);
            self.gl.enable(glow::CULL_FACE);
            self.gl.cull_face(glow::BACK);
            self.gl.front_face(glow::CCW);
            self.gl.viewport(0, 0, width as i32, height as i32);
        }

        RenderSystem::update(&self.gl, width, height);
        PhysicsSystem::update();

        unsafe {
            self.gl.viewport(viewport[0], viewport[1], viewport[2], viewport[3]);
            self.gl.disable(glow::DEPTH_TEST);
            self.gl.depth_func(depth_func as u32);
            self.gl.depth_mask(writemask != 0);
            self.gl.disable(glow::CULL_FACE);
            self.gl.disable(glow::BLEND);
            // Note: Skipping program restoration as it requires proper OpenGL program handle management
            self.gl.clear(glow::DEPTH_BUFFER_BIT);
        }
    }
}
