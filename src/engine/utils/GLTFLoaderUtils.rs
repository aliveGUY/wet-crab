use gltf::buffer::Data;
use glow::HasContext;
use crate::index::engine::components::SharedComponents::{Mesh, Material};
use crate::index::engine::components::AnimatedObject3D::{Skeleton, Node, AnimationChannel, AnimationType};
use crate::index::engine::utils::math::mat4x4_transpose;

pub fn extract_mesh(
    gl: &glow::Context,
    gltf: &gltf::Gltf,
    buffers: &[Data],
    asset_name: &str
) -> Mesh {
    let primitive = gltf
        .meshes()
        .next()
        .unwrap_or_else(|| panic!("No mesh found for {:?}", asset_name))
        .primitives()
        .next()
        .unwrap_or_else(|| panic!("No primitive found for {:?}", asset_name));

    macro_rules! extract {
        ($sem:expr, $ty:ty) => {
            extract_buffer_data::<$ty>(&buffers, &primitive.get(&$sem)
                .unwrap_or_else(|| panic!("Missing {} for {:?}", stringify!($sem), asset_name)))
                .unwrap_or_else(|e| panic!("Failed to extract {} for {:?}: {}", stringify!($sem), asset_name, e))
        };
    }

    macro_rules! extract_optional {
        ($sem:expr, $ty:ty) => {
            primitive.get(&$sem)
                .and_then(|accessor| extract_buffer_data::<$ty>(&buffers, &accessor).ok())
        };
    }

    // Extract basic mesh data (always required)
    let positions: Vec<f32> = extract!(gltf::Semantic::Positions, f32);
    let normals: Vec<f32> = extract!(gltf::Semantic::Normals, f32);
    let tex_coords: Vec<f32> = extract!(gltf::Semantic::TexCoords(0), f32);
    let indices: Vec<u16> = extract_buffer_data(
        &buffers,
        &primitive.indices().unwrap_or_else(|| panic!("No indices found for {:?}", asset_name))
    ).unwrap_or_else(|e| panic!("Failed to extract indices for {:?}: {}", asset_name, e));

    // Extract skeletal data (optional - only for animated meshes)
    let joints: Option<Vec<u8>> = extract_optional!(gltf::Semantic::Joints(0), u8);
    let weights: Option<Vec<f32>> = extract_optional!(gltf::Semantic::Weights(0), f32);

    let has_skeletal_data = joints.is_some() && weights.is_some();

    unsafe {
        let vao = gl.create_vertex_array()
            .unwrap_or_else(|e| panic!("Failed to create VAO for {:?}: {}", asset_name, e));
        gl.bind_vertex_array(Some(vao));

        let setup_attrib = |loc, data: &[u8], size, ty, stride, int| {
            let buf = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(buf));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data, glow::STATIC_DRAW);
            gl.enable_vertex_attrib_array(loc);
            if int {
                gl.vertex_attrib_pointer_i32(loc, size, ty, stride, 0);
            } else {
                gl.vertex_attrib_pointer_f32(loc, size, ty, false, stride, 0);
            }
        };

        // Set up basic mesh attributes (always present)
        setup_attrib(1, bytemuck::cast_slice(&positions), 3, glow::FLOAT, 12, false);  // Position
        setup_attrib(0, bytemuck::cast_slice(&normals), 3, glow::FLOAT, 12, false);    // Normal
        setup_attrib(4, bytemuck::cast_slice(&tex_coords), 2, glow::FLOAT, 8, false);  // TexCoord

        // Set up skeletal attributes (only if present)
        if has_skeletal_data {
            if let (Some(joints_data), Some(weights_data)) = (joints, weights) {
                setup_attrib(2, &joints_data, 4, glow::UNSIGNED_BYTE, 4, true);           // Joints
                setup_attrib(3, bytemuck::cast_slice(&weights_data), 4, glow::FLOAT, 16, false); // Weights
            }
        }

        let ebo = gl.create_buffer()
            .unwrap_or_else(|e| panic!("Failed to create EBO for {:?}: {}", asset_name, e));
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            bytemuck::cast_slice(&indices),
            glow::STATIC_DRAW
        );

        gl.bind_vertex_array(None);

        Mesh {
            vao,
            index_count: indices.len(),
            vertex_count: positions.len() / 3,
        }
    }
}

pub fn extract_skeleton(
    gltf: &gltf::Gltf,
    buffers: &[Data],
    asset_name: &str
) -> Skeleton {
    let mut node_parents = vec![u32::MAX; gltf.nodes().len()];
    for node in gltf.nodes() {
        for child in node.children() {
            node_parents[child.index()] = node.index() as u32;
        }
    }

    let nodes = gltf
        .nodes()
        .map(|n| {
            let (t, r, s) = n.transform().decomposed();
            Node {
                translation: t,
                rotation: r,
                scale: s,
                parent: node_parents[n.index()],
            }
        })
        .collect::<Vec<_>>();

    let (joint_ids, joint_inverse_mats) = if let Some(skin) = gltf.skins().next() {
        let ids = skin
            .joints()
            .map(|j| j.index() as u32)
            .collect();
        let mut inv_mats = Vec::new();
        if let Some(ibm) = skin.inverse_bind_matrices() {
            let data: Vec<f32> = extract_buffer_data(&buffers, &ibm)
                .unwrap_or_else(|e| panic!("Failed to extract inverse bind matrices for {:?}: {}", asset_name, e));
            inv_mats = data
                .chunks(16)
                .map(|m| {
                    let mut mat = [0.0; 16];
                    mat.copy_from_slice(m);
                    mat4x4_transpose(mat)
                })
                .collect();
        }
        (ids, inv_mats)
    } else {
        panic!("No skeleton/skin found for animated asset {:?}", asset_name);
    };

    if nodes.is_empty() {
        panic!("No nodes found for skeleton in {:?}", asset_name);
    }

    Skeleton {
        nodes,
        joint_ids,
        joint_inverse_mats,
    }
}

pub fn extract_animation_channels(gltf: &gltf::Gltf, buffers: &[Data], asset_name: &str) -> Vec<AnimationChannel> {
    gltf.animations()
        .next()
        .map(|anim| {
            anim.channels()
                .filter_map(|chan| {
                    let anim_type = match chan.target().property() {
                        gltf::animation::Property::Translation => AnimationType::Translation,
                        gltf::animation::Property::Rotation => AnimationType::Rotation,
                        gltf::animation::Property::Scale => AnimationType::Scale,
                        _ => {
                            return None;
                        }
                    };

                    let times = extract_buffer_data::<f32>(&buffers, &chan.sampler().input()).ok()?;
                    let data = extract_buffer_data::<f32>(&buffers, &chan.sampler().output()).ok()?;

                    Some(AnimationChannel {
                        target: chan.target().node().index() as u32,
                        animation_type: anim_type,
                        num_timesteps: times.len(),
                        times,
                        data,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

use image::io::Reader as ImageReader;
use std::io::Cursor;

// Proper PNG decoder using the image crate
fn decode_png_with_crate(png_data: &[u8]) -> Result<(u32, u32, Vec<u8>), Box<dyn std::error::Error>> {
    let img = ImageReader::new(Cursor::new(png_data))
        .with_guessed_format()?
        .decode()?;
    
    println!("🔍 Original image format: {:?}", img.color());
    
    let rgba_img = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();
    let pixels = rgba_img.into_raw();
    
    // Debug: Sample some pixel values to see what we're getting
    if pixels.len() >= 16 {
        println!("🎨 Sample pixels (RGBA):");
        for i in 0..4 {
            let idx = i * 4;
            println!("  Pixel {}: R={}, G={}, B={}, A={}", 
                i, pixels[idx], pixels[idx+1], pixels[idx+2], pixels[idx+3]);
        }
    }
    
    // Check for pure black pixels
    let mut black_pixel_count = 0;
    let mut total_pixels = 0;
    for chunk in pixels.chunks(4) {
        total_pixels += 1;
        if chunk[0] == 0 && chunk[1] == 0 && chunk[2] == 0 {
            black_pixel_count += 1;
        }
    }
    println!("🖤 Black pixels found: {}/{} ({:.1}%)", 
        black_pixel_count, total_pixels, 
        (black_pixel_count as f32 / total_pixels as f32) * 100.0);
    
    Ok((width, height, pixels))
}

// Helper function to determine if GLTF has skeletal data
pub fn has_skeletal_data(gltf: &gltf::Gltf) -> bool {
    // Check if any mesh primitive has joints and weights
    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            if primitive.get(&gltf::Semantic::Joints(0)).is_some() && 
               primitive.get(&gltf::Semantic::Weights(0)).is_some() {
                return true;
            }
        }
    }
    false
}

pub fn extract_material(
    gl: &glow::Context,
    gltf: &gltf::Gltf,
    _buffers: &[Data],
    png_data: &[u8],
    shader_program: glow::Program,
    asset_name: &str
) -> Material {
    let material = gltf.materials().next()
        .unwrap_or_else(|| panic!("No material found for {:?}", asset_name));
    
    let pbr = material.pbr_metallic_roughness();
    
    let mut mat = Material::new(shader_program);
    mat.metallic_factor = pbr.metallic_factor();
    mat.roughness_factor = pbr.roughness_factor();
    mat.double_sided = material.double_sided();

    // Extract texture if present
    if let Some(base_color_info) = pbr.base_color_texture() {
        let texture_index = base_color_info.texture().index();
        if let Some(texture) = gltf.textures().nth(texture_index) {
            if let Some(_image) = gltf.images().nth(texture.source().index()) {
                
                match decode_png_with_crate(png_data) {
                    Ok((width, height, rgba_pixels)) => {
                        unsafe {
                            let gl_texture = gl.create_texture()
                                .unwrap_or_else(|e| panic!("Failed to create texture for {:?}: {}", asset_name, e));
                            gl.bind_texture(glow::TEXTURE_2D, Some(gl_texture));
                            
                            gl.tex_image_2d(
                                glow::TEXTURE_2D,
                                0,
                                glow::RGBA as i32,
                                width as i32,
                                height as i32,
                                0,
                                glow::RGBA,
                                glow::UNSIGNED_BYTE,
                                glow::PixelUnpackData::Slice(Some(&rgba_pixels))
                            );
                            
                            // Set texture parameters
                            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
                            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
                            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
                            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
                            
                            gl.bind_texture(glow::TEXTURE_2D, None);
                            
                            mat.base_color_texture = Some(gl_texture);
                            
                            println!("✅ Texture loaded: {}x{} pixels", width, height);
                        }
                    }
                    Err(e) => {
                        panic!("Failed to decode PNG for {:?}: {}", asset_name, e);
                    }
                }
            }
        }
    }

    mat
}

pub fn extract_buffer_data<T: bytemuck::Pod>(
    buffers: &[Data],
    accessor: &gltf::Accessor
) -> Result<Vec<T>, Box<dyn std::error::Error>> {
    let view = accessor.view().ok_or("Missing buffer view")?;
    let buffer = &buffers[view.buffer().index()];
    let start = view.offset() + accessor.offset();
    let end = start + accessor.count() * accessor.size();

    if end > buffer.len() {
        return Err("Buffer overflow".into());
    }

    let slice = &buffer[start..end];
    let typed_slice = bytemuck::cast_slice(slice);
    Ok(typed_slice.to_vec())
}
