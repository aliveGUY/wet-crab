use std::collections::HashMap;
use once_cell::sync::Lazy;

// Import required components - using the module path from index.rs
use crate::index::object3d::Object3D;
use crate::index::gltf_loader_utils::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Assets {
    TestingDoll,
    Chair,
}

struct AssetsManager {
    assets: HashMap<Assets, Object3D>,
    initialized: bool,
}

impl AssetsManager {
    fn new() -> Self {
        Self {
            assets: HashMap::new(),
            initialized: false,
        }
    }

    fn initialize(&mut self, gl: &glow::Context) {
        if self.initialized {
            println!("⚠️  AssetsManager already initialized");
            return;
        }

        println!("🔄 Initializing AssetsManager and loading all assets...");

        self.load_gltf(
            include_str!("../../assets/meshes/guy.gltf"),
            include_bytes!("../../assets/meshes/guy.bin"),
            include_bytes!("../../assets/textures/Material Base Color.png"),
            Assets::TestingDoll,
            gl
        );

        self.load_gltf(
            include_str!("../../assets/meshes/chair.gltf"),
            include_bytes!("../../assets/meshes/chair.bin"),
            include_bytes!("../../assets/textures/wood-texture.png"),
            Assets::Chair,
            gl
        );

        self.initialized = true;
        println!("✅ AssetsManager initialization complete. Loaded {} assets.", self.assets.len());
    }

    fn get_object3d_copy(&self, asset_name: Assets) -> Object3D {
        if !self.initialized {
            eprintln!(
                "❌ AssetsManager not initialized! Call initialize() first. Using default Object3D."
            );
            return Object3D::new();
        }

        // Simply get copy from map - no file access, no GL context needed
        if let Some(object3d) = self.assets.get(&asset_name) {
            println!("✅ Retrieved copy of asset: {:?} from cache", asset_name);
            object3d.clone()
        } else {
            eprintln!("❌ Asset {:?} not found in cache, using default Object3D", asset_name);
            Object3D::new()
        }
    }

    fn load_gltf(
        &mut self,
        gltf_data: &str,
        bin_data: &[u8],
        png_data: &[u8],
        asset_name: Assets,
        gl: &glow::Context
    ) {
        println!("🔄 Loading GLTF asset: {:?}", asset_name);

        // Parse asset data ONCE during initialization
        let gltf = match gltf::Gltf::from_slice(gltf_data.as_bytes()) {
            Ok(gltf) => gltf,
            Err(e) => {
                eprintln!("⚠️  Failed to parse GLTF for {:?}: {}", asset_name, e);
                return;
            }
        };
        let buffers = vec![gltf::buffer::Data(bin_data.to_vec())];

        // Create Object3D with GPU resources
        let mesh = match extract_mesh(gl, &gltf, &buffers) {
            Ok(mesh) => mesh,
            Err(e) => {
                eprintln!("⚠️  Failed to extract mesh for {:?}: {}", asset_name, e);
                return;
            }
        };

        let material = match extract_material(gl, &gltf, &buffers, png_data) {
            Ok(material) => material,
            Err(e) => {
                eprintln!("⚠️  Failed to extract material for {:?}: {}", asset_name, e);
                return;
            }
        };

        let skeleton = match extract_skeleton(&gltf, &buffers) {
            Ok(skeleton) => skeleton,
            Err(e) => {
                eprintln!("⚠️  Failed to extract skeleton for {:?}: {}", asset_name, e);
                return;
            }
        };

        let animation_channels = extract_animation_channels(&gltf, &buffers);

        let mut object3d = Object3D::with_mesh(mesh);

        if let Some(mat) = material {
            object3d.set_material(mat);
        }

        if let Some(skel) = skeleton {
            object3d.set_skeleton(skel);
        }

        object3d.set_animation_channels(animation_channels);

        // Store complete Object3D in map
        self.assets.insert(asset_name, object3d);
        println!("✅ Loaded and cached asset: {:?}", asset_name);
    }
}

// Global singleton instance
static ASSETS_MANAGER: Lazy<std::sync::Mutex<AssetsManager>> = Lazy::new(|| {
    std::sync::Mutex::new(AssetsManager::new())
});

// Public API - Only two methods as requested
pub fn initialize(gl: &glow::Context) {
    ASSETS_MANAGER.lock().unwrap().initialize(gl)
}

pub fn get_object3d_copy(asset_name: Assets) -> Object3D {
    ASSETS_MANAGER.lock().unwrap().get_object3d_copy(asset_name)
}
