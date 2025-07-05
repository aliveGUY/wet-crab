use gltf::buffer::Data;

mod gltf_loader_utils {
    use super::*;
    include!("../utils/GLTFLoaderUtils.rs");
}
use gltf_loader_utils::*;

pub fn load_model(gl: &glow::Context) -> Result<Object3D, Box<dyn std::error::Error>> {
    println!("🔄 Loading embedded GLTF data...");

    let gltf = gltf::Gltf::from_slice(include_str!("../../assets/meshes/guy.gltf").as_bytes())?;
    let buffers = vec![gltf::buffer::Data(include_bytes!("../../assets/meshes/guy.bin").to_vec())];

    let mesh = extract_mesh(gl, &gltf, &buffers)?;
    let skeleton = extract_skeleton(&gltf, &buffers)?;
    let animation_channels = extract_animation_channels(&gltf, &buffers);

    let mut object3d = Object3D::with_mesh(mesh);

    if let Some(skel) = skeleton {
        object3d.set_skeleton(skel);
    }

    object3d.set_animation_channels(animation_channels);

    println!(
        "✅ Model loaded: {} nodes, {} animations, {} joints",
        object3d.skeleton.as_ref().map_or(0, |s| s.nodes.len()),
        object3d.animation_channels.len(),
        object3d.skeleton.as_ref().map_or(0, |s| s.joint_ids.len())
    );

    Ok(object3d)
}
