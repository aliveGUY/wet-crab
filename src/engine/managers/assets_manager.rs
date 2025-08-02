use std::collections::HashMap;
use std::cell::RefCell;
use glow::HasContext;

// Import required components - using the new module structure
use crate::index::engine::components::{ StaticObject3DComponent, AnimatedObject3DComponent };
use crate::index::engine::components::SharedComponents::{ Transform };
use crate::index::engine::utils::gltf_loader_utils::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Assets {
    TestingDoll,
    Chair,
    BlockoutPlatform,
}

pub struct AssetsManager {
    static_assets: HashMap<Assets, StaticObject3DComponent>,
    animated_assets: HashMap<Assets, AnimatedObject3DComponent>,
    static_shader_program: Option<glow::Program>,
    animated_shader_program: Option<glow::Program>,
    static_outline_shader_program: Option<glow::Program>,
    animated_outline_shader_program: Option<glow::Program>,
    box_shader_program: Option<glow::Program>,
    sphere_shader_program: Option<glow::Program>,
    capsule_shader_program: Option<glow::Program>,
    cylinder_shader_program: Option<glow::Program>,
    initialized: bool,
}

impl AssetsManager {
    fn new() -> Self {
        Self {
            static_assets: HashMap::new(),
            animated_assets: HashMap::new(),
            static_shader_program: None,
            animated_shader_program: None,
            static_outline_shader_program: None,
            animated_outline_shader_program: None,
            box_shader_program: None,
            sphere_shader_program: None,
            capsule_shader_program: None,
            cylinder_shader_program: None,
            initialized: false,
        }
    }

    fn initialize_asset_manager(&mut self, gl: &glow::Context) {
        if self.initialized {
            println!("⚠️  AssetsManager already initialized");
            return;
        }

        println!("🔄 Initializing AssetsManager and loading all assets...");

        // Create shader programs first
        let static_shader = create_shader_program(
            gl,
            include_str!("../../assets/shaders/vertex_static.glsl"),
            include_str!("../../assets/shaders/fragment_static.glsl"),
            "static"
        );
        let animated_shader = create_shader_program(
            gl,
            include_str!("../../assets/shaders/vertex_animated.glsl"),
            include_str!("../../assets/shaders/fragment_animated.glsl"),
            "animated"
        );

        // Create outline shader programs
        let static_outline_shader = create_shader_program(
            gl,
            include_str!("../../assets/shaders/vertex_outline_static.glsl"),
            include_str!("../../assets/shaders/fragment_outline.glsl"),
            "static_outline"
        );
        let animated_outline_shader = create_shader_program(
            gl,
            include_str!("../../assets/shaders/vertex_outline_animated.glsl"),
            include_str!("../../assets/shaders/fragment_outline.glsl"),
            "animated_outline"
        );

        // Create shape-specific shader programs
        let box_shader = create_shader_program(
            gl,
            include_str!("../../assets/shaders/vertex_box.glsl"),
            include_str!("../../assets/shaders/fragment_box.glsl"),
            "box"
        );
        let sphere_shader = create_shader_program(
            gl,
            include_str!("../../assets/shaders/vertex_sphere.glsl"),
            include_str!("../../assets/shaders/fragment_sphere.glsl"),
            "sphere"
        );
        let capsule_shader = create_shader_program(
            gl,
            include_str!("../../assets/shaders/vertex_capsule.glsl"),
            include_str!("../../assets/shaders/fragment_capsule.glsl"),
            "capsule"
        );
        let cylinder_shader = create_shader_program(
            gl,
            include_str!("../../assets/shaders/vertex_cylinder.glsl"),
            include_str!("../../assets/shaders/fragment_cylinder.glsl"),
            "cylinder"
        );

        self.static_shader_program = Some(static_shader);
        self.animated_shader_program = Some(animated_shader);
        self.static_outline_shader_program = Some(static_outline_shader);
        self.animated_outline_shader_program = Some(animated_outline_shader);
        self.box_shader_program = Some(box_shader);
        self.sphere_shader_program = Some(sphere_shader);
        self.capsule_shader_program = Some(capsule_shader);
        self.cylinder_shader_program = Some(cylinder_shader);

        // Load animated asset (TestingDoll)
        self.load_animated_gltf(
            include_str!("../../assets/meshes/guy.gltf"),
            include_bytes!("../../assets/meshes/guy.bin"),
            include_bytes!("../../assets/textures/Material Base Color.png"),
            Assets::TestingDoll,
            animated_shader,
            gl
        );

        // Load static asset (Chair)
        self.load_static_gltf(
            include_str!("../../assets/meshes/chair.gltf"),
            include_bytes!("../../assets/meshes/chair.bin"),
            include_bytes!("../../assets/textures/wood-texture.png"),
            Assets::Chair,
            static_shader,
            gl
        );

        self.load_static_gltf(
            include_str!("../../assets/meshes/blockout_platform.gltf"),
            include_bytes!("../../assets/meshes/blockout_platform.bin"),
            include_bytes!("../../assets/textures/orange-blueprint.png"),
            Assets::BlockoutPlatform,
            static_shader,
            gl
        );

        self.initialized = true;
        let total_assets = self.static_assets.len() + self.animated_assets.len();
        println!("✅ AssetsManager initialization complete. Loaded {} assets.", total_assets);
    }

    pub fn get_static_object_copy(&self, asset_name: Assets) -> StaticObject3DComponent {
        if !self.initialized {
            panic!("❌ AssetsManager not initialized! Call initialize_asset_manager() first.");
        }

        if let Some(object) = self.static_assets.get(&asset_name) {
            println!("✅ Retrieved static copy of asset: {:?} from cache", asset_name);
            object.clone()
        } else {
            panic!("❌ Static asset {:?} not found in cache", asset_name);
        }
    }

    pub fn get_animated_object_copy(&self, asset_name: Assets) -> AnimatedObject3DComponent {
        if !self.initialized {
            panic!("❌ AssetsManager not initialized! Call initialize_asset_manager() first.");
        }

        if let Some(object) = self.animated_assets.get(&asset_name) {
            println!("✅ Retrieved animated copy of asset: {:?} from cache", asset_name);
            object.clone()
        } else {
            panic!("❌ Animated asset {:?} not found in cache", asset_name);
        }
    }

    fn load_static_gltf(
        &mut self,
        gltf_data: &str,
        bin_data: &[u8],
        png_data: &[u8],
        asset_name: Assets,
        shader_program: glow::Program,
        gl: &glow::Context
    ) {
        println!("🔄 Loading static GLTF asset: {:?}", asset_name);

        // Parse asset data
        let gltf = gltf::Gltf
            ::from_slice(gltf_data.as_bytes())
            .unwrap_or_else(|e| panic!("Failed to parse GLTF for {:?}: {}", asset_name, e));
        let buffers = vec![gltf::buffer::Data(bin_data.to_vec())];

        // Extract components - all error handling is internal
        let asset_name_str = format!("{:?}", asset_name);
        let mesh = extract_mesh(gl, &gltf, &buffers, &asset_name_str);
        let material = extract_material(
            gl,
            &gltf,
            &buffers,
            png_data,
            shader_program,
            &asset_name_str
        );

        // Create static object with default transform
        let mut transform = Transform::new(0.0, 0.0, 0.0);
        transform.translate(0.0, 0.0, 0.0); // Default position

        let static_object = StaticObject3DComponent::new(mesh, material, asset_name);

        // Store in static assets map
        self.static_assets.insert(asset_name, static_object);
        println!("✅ Loaded and cached static asset: {:?}", asset_name);
    }

    fn load_animated_gltf(
        &mut self,
        gltf_data: &str,
        bin_data: &[u8],
        png_data: &[u8],
        asset_name: Assets,
        shader_program: glow::Program,
        gl: &glow::Context
    ) {
        println!("🔄 Loading animated GLTF asset: {:?}", asset_name);

        // Parse asset data
        let gltf = gltf::Gltf
            ::from_slice(gltf_data.as_bytes())
            .unwrap_or_else(|e| panic!("Failed to parse GLTF for {:?}: {}", asset_name, e));
        let buffers = vec![gltf::buffer::Data(bin_data.to_vec())];

        // Extract components - all error handling is internal
        let asset_name_str = format!("{:?}", asset_name);
        let mesh = extract_mesh(gl, &gltf, &buffers, &asset_name_str);
        let material = extract_material(
            gl,
            &gltf,
            &buffers,
            png_data,
            shader_program,
            &asset_name_str
        );
        let skeleton = extract_skeleton(&gltf, &buffers, &asset_name_str);
        let animation_channels = extract_animation_channels(&gltf, &buffers, &asset_name_str);

        // Create animated object with default transform
        let mut transform = Transform::new(0.0, 0.0, 0.0);
        transform.translate(0.0, 0.0, 0.0); // Default position

        let animated_object = AnimatedObject3DComponent::new(
            mesh,
            material,
            skeleton,
            animation_channels,
            asset_name
        );

        // Store in animated assets map
        self.animated_assets.insert(asset_name, animated_object);
        println!("✅ Loaded and cached animated asset: {:?}", asset_name);
    }
}

// Shader creation functions
fn compile_shader(
    gl: &glow::Context,
    shader_type: u32,
    source: String
) -> Result<glow::Shader, String> {
    unsafe {
        let shader = gl.create_shader(shader_type)?;
        gl.shader_source(shader, &source);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            let log = gl.get_shader_info_log(shader);
            gl.delete_shader(shader);
            return Err(format!("Shader compile error: {}", log));
        }
        Ok(shader)
    }
}

fn create_shader_program(
    gl: &glow::Context,
    vertex_shader_source: &str,
    fragment_shader_source: &str,
    program_name: &str
) -> glow::Program {
    unsafe {
        // Use shader sources directly (no version replacement needed)
        let vs = compile_shader(
            gl,
            glow::VERTEX_SHADER,
            vertex_shader_source.to_string()
        ).unwrap_or_else(|e| panic!("Failed to compile {} vertex shader: {}", program_name, e));
        let fs = compile_shader(
            gl,
            glow::FRAGMENT_SHADER,
            fragment_shader_source.to_string()
        ).unwrap_or_else(|e| panic!("Failed to compile {} fragment shader: {}", program_name, e));

        let program = gl
            .create_program()
            .unwrap_or_else(|e| panic!("Failed to create {} shader program: {}", program_name, e));
        gl.attach_shader(program, vs);
        gl.attach_shader(program, fs);
        gl.link_program(program);

        if !gl.get_program_link_status(program) {
            let log = gl.get_program_info_log(program);
            panic!("{} shader program link error: {}", program_name, log);
        }

        gl.delete_shader(vs);
        gl.delete_shader(fs);

        println!("✅ Created {} shader program", program_name);
        program
    }
}

// Global singleton instance - single-threaded
thread_local! {
    static ASSETS_MANAGER: RefCell<AssetsManager> = RefCell::new(AssetsManager::new());
}

// Public API
pub fn initialize_asset_manager(gl: &glow::Context) {
    ASSETS_MANAGER.with(|manager| { manager.borrow_mut().initialize_asset_manager(gl) })
}

pub fn get_static_object_copy(asset_name: Assets) -> StaticObject3DComponent {
    ASSETS_MANAGER.with(|manager| { manager.borrow().get_static_object_copy(asset_name) })
}

pub fn get_animated_object_copy(asset_name: Assets) -> AnimatedObject3DComponent {
    ASSETS_MANAGER.with(|manager| { manager.borrow().get_animated_object_copy(asset_name) })
}

pub fn get_static_outline_shader() -> glow::Program {
    ASSETS_MANAGER.with(|manager| {
        manager.borrow().static_outline_shader_program
            .expect("Static outline shader not initialized")
    })
}

pub fn get_animated_outline_shader() -> glow::Program {
    ASSETS_MANAGER.with(|manager| {
        manager.borrow().animated_outline_shader_program
            .expect("Animated outline shader not initialized")
    })
}

pub fn get_box_shader() -> glow::Program {
    ASSETS_MANAGER.with(|manager| {
        manager.borrow().box_shader_program
            .expect("Box shader not initialized")
    })
}

pub fn get_sphere_shader() -> glow::Program {
    ASSETS_MANAGER.with(|manager| {
        manager.borrow().sphere_shader_program
            .expect("Sphere shader not initialized")
    })
}

pub fn get_capsule_shader() -> glow::Program {
    ASSETS_MANAGER.with(|manager| {
        manager.borrow().capsule_shader_program
            .expect("Capsule shader not initialized")
    })
}

pub fn get_cylinder_shader() -> glow::Program {
    ASSETS_MANAGER.with(|manager| {
        manager.borrow().cylinder_shader_program
            .expect("Cylinder shader not initialized")
    })
}
