pub fn extract_mesh(
    gl: &glow::Context,
    gltf: &gltf::Gltf,
    buffers: &[Data]
) -> Result<Mesh, Box<dyn std::error::Error>> {
    let primitive = gltf
        .meshes()
        .next()
        .ok_or("No mesh found")?
        .primitives()
        .next()
        .ok_or("No primitive found")?;

    macro_rules! extract {
        ($sem:expr, $ty:ty) => {
            extract_buffer_data::<$ty>(&buffers, &primitive.get(&$sem).ok_or(concat!("Missing ", stringify!($sem)))?)?
        };
    }

    let positions: Vec<f32> = extract!(gltf::Semantic::Positions, f32);
    let normals: Vec<f32> = extract!(gltf::Semantic::Normals, f32);
    let joints: Vec<u8> = extract!(gltf::Semantic::Joints(0), u8);
    let weights: Vec<f32> = extract!(gltf::Semantic::Weights(0), f32);
    let indices: Vec<u16> = extract_buffer_data(
        &buffers,
        &primitive.indices().ok_or("No indices")?
    )?;

    unsafe {
        let vao = gl.create_vertex_array()?;
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

        setup_attrib(1, bytemuck::cast_slice(&positions), 3, glow::FLOAT, 12, false);
        setup_attrib(0, bytemuck::cast_slice(&normals), 3, glow::FLOAT, 12, false);
        setup_attrib(2, &joints, 4, glow::UNSIGNED_BYTE, 4, true);
        setup_attrib(3, bytemuck::cast_slice(&weights), 4, glow::FLOAT, 16, false);

        let ebo = gl.create_buffer()?;
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            bytemuck::cast_slice(&indices),
            glow::STATIC_DRAW
        );

        gl.bind_vertex_array(None);

        Ok(Mesh {
            vao,
            index_count: indices.len(),
            vertex_count: positions.len() / 3,
        })
    }
}

pub fn extract_skeleton(
    gltf: &gltf::Gltf,
    buffers: &[Data]
) -> Result<Option<Skeleton>, Box<dyn std::error::Error>> {
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
            let data: Vec<f32> = extract_buffer_data(&buffers, &ibm)?;
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
        (Vec::new(), Vec::new())
    };

    Ok(
        if !nodes.is_empty() {
            Some(Skeleton {
                nodes,
                joint_ids,
                joint_inverse_mats,
            })
        } else {
            None
        }
    )
}

pub fn extract_animation_channels(gltf: &gltf::Gltf, buffers: &[Data]) -> Vec<AnimationChannel> {
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
