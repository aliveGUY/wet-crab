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

pub static PLAYER_ENTITY_ID: Lazy<RwLock<Option<EntityId>>> = Lazy::new(|| RwLock::new(None));

pub struct Program {
    gl: Arc<glow::Context>,
    render_pass_manager: RenderPassManager,
    width: u32,
    height: u32,
}

impl Program {
    pub fn new(gl: glow::Context) -> Result<Self, String> {
        initialize_asset_manager(&gl);

        let chair_entity_id = spawn();
        insert_many!(
            chair_entity_id,
            get_static_object_copy(Assets::Chair),
            TransformComponent::with_translation(2.0, -3.0, -5.0),
            ColliderComponent::new()
        );

        let doll_entity_id = spawn();
        insert_many!(
            doll_entity_id,
            get_animated_object_copy(Assets::TestingDoll),
            TransformComponent::with_translation(-2.0, -3.0, -5.0)
        );

        let player_entity_id = spawn();
        *PLAYER_ENTITY_ID.write().unwrap() = Some(player_entity_id.clone());
        insert_many!(player_entity_id, CameraComponent::new());

        EventSystem::subscribe(EventType::Move, Arc::new(MovementSystem));
        EventSystem::subscribe(EventType::RotateCamera, Arc::new(CameraRotationSystem));

        let gl_arc = Arc::new(gl);
        
        // Create UI manager
        let ui_manager = UIManager::new(gl_arc.clone())?;
        
        // Create render pass manager
        let mut render_pass_manager = RenderPassManager::new(gl_arc.clone());
        
        // Add 3D scene rendering pass (renders first)
        render_pass_manager.add_pass(Box::new(Scene3DPass::new()));
        
        // Add GUI rendering pass (renders last, on top of 3D scene)
        render_pass_manager.add_pass(Box::new(GuiPass::new(ui_manager)));

        unsafe {
            gl_arc.enable(glow::DEPTH_TEST);
        }

        println!("✅ Program initialized successfully with ECS-based architecture");
        println!("🎨 Render passes configured: {:?}", render_pass_manager.get_pass_names());

        Ok(Self {
            gl: gl_arc,
            render_pass_manager,
            width: 0,
            height: 0,
        })
    }

    pub fn render(&mut self, width: u32, height: u32, _delta_time: f32) -> Result<(), String> {
        self.width = width;
        self.height = height;

        // Update game systems (non-rendering)
        ColliderSystem::update();

        // Execute all rendering passes with proper state isolation
        // This will render 3D scene first, then GUI on top
        self.render_pass_manager.execute_passes(width, height);

        Ok(())
    }

    #[allow(dead_code)]
    pub fn cleanup(&self) {
        println!("Program cleanup completed");
    }
}
