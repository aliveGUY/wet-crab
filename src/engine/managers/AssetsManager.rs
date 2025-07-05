use std::collections::HashMap;
use once_cell::sync::Lazy;

// Import required components - using the module path from index.rs
use crate::index::object3d::Object3D;
use crate::index::gltf_loader_utils::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Assets {
    TestingDoll,
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

    fn initialize(&mut self, gl: &glow::Context) -> Result<(), Box<dyn std::error::Error>> {
        if self.initialized {
            println!("⚠️  AssetsManager already initialized");
            return Ok(());
        }

        println!("🔄 Initializing AssetsManager and loading all assets...");
        
        // Load TestingDoll asset once
        self.loadGltf(
            "../../assets/meshes/guy.gltf",
            "../../assets/meshes/guy.bin", 
            "../../assets/textures/Material Base Color.png",
            Assets::TestingDoll,
            gl
        )?;
        
        self.initialized = true;
        println!("✅ AssetsManager initialization complete. Loaded {} assets.", self.assets.len());
        Ok(())
    }

    fn getObject3DCopy(&self, assetName: Assets) -> Option<Object3D> {
        if !self.initialized {
            eprintln!("❌ AssetsManager not initialized! Call initialize() first.");
            return None;
        }

        // Simply get copy from map - no file access, no GL context needed
        if let Some(object3d) = self.assets.get(&assetName) {
            println!("✅ Retrieved copy of asset: {:?} from cache", assetName);
            Some(object3d.clone())
        } else {
            eprintln!("❌ Asset {:?} not found in cache", assetName);
            None
        }
    }

    fn loadGltf(
        &mut self,
        gltfPath: &str,
        binPath: &str,
        texturePath: &str,
        assetName: Assets,
        gl: &glow::Context
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔄 Loading GLTF from paths: {}, {}, {}", gltfPath, binPath, texturePath);
        
        // Load and parse asset files ONCE during initialization
        let gltf_data = include_str!("../../assets/meshes/guy.gltf");
        let gltf = gltf::Gltf::from_slice(gltf_data.as_bytes())?;
        let bin_data = include_bytes!("../../assets/meshes/guy.bin");
        let buffers = vec![gltf::buffer::Data(bin_data.to_vec())];
        let png_data = include_bytes!("../../assets/textures/Material Base Color.png");

        // Create Object3D with GPU resources
        let mesh = extract_mesh(gl, &gltf, &buffers)?;
        let material = extract_material(gl, &gltf, &buffers, png_data)?;
        let skeleton = extract_skeleton(&gltf, &buffers)?;
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
        self.assets.insert(assetName, object3d);
        println!("✅ Loaded and cached asset: {:?}", assetName);
        Ok(())
    }
}

// Global singleton instance
static ASSETS_MANAGER: Lazy<std::sync::Mutex<AssetsManager>> = Lazy::new(|| {
    std::sync::Mutex::new(AssetsManager::new())
});

// Public API - Only two methods as requested
pub fn initialize(gl: &glow::Context) -> Result<(), Box<dyn std::error::Error>> {
    ASSETS_MANAGER.lock().unwrap().initialize(gl)
}

pub fn getObject3DCopy(assetName: Assets) -> Option<Object3D> {
    ASSETS_MANAGER.lock().unwrap().getObject3DCopy(assetName)
}
