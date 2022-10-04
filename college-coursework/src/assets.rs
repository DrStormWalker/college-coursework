use std::io::{BufReader, Cursor, Error};

use wgpu::util::DeviceExt;

use crate::renderer::{model, texture};

pub async fn load_string(file_name: &str) -> Result<String, Error> {
    //! Loads the contents of an asset into a string from the file system

    // Get the path relative to the assets directory
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("assets")
        .join(file_name);

    log::info!("Loading {:?} as string", file_name);

    // Open and read the file
    let txt = std::fs::read_to_string(path)?;
    Ok(txt)
}

pub async fn load_binary(file_name: &str) -> Result<Vec<u8>, Error> {
    //! Loads the contents of an asset into a byte vector from the file syste,

    // get the path relative to the assets directory
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("assets")
        .join(file_name);

    log::info!("Loading {:?} as binary data", file_name);

    // Open and read the file
    let data = std::fs::read(path)?;
    Ok(data)
}

/// Possible errors produced by the load_texture function
#[derive(thiserror::Error, Debug)]
pub enum LoadTextureError {
    #[error(transparent)]
    IoError(#[from] Error),

    #[error(transparent)]
    ImageError(#[from] image::ImageError),
}

pub async fn load_texture(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> Result<texture::Texture, LoadTextureError> {
    //! Loads the contents of an asset into a texture object from the file system

    log::info!("Loading {:?} as a texture", file_name);

    // Load the data from the file as binary data
    let data = load_binary(file_name).await?;

    // Use the binary data to create a texture
    Ok(texture::Texture::from_bytes(
        device, queue, &data, file_name,
    )?)
}

/// Possible errors produced by the load_model function
#[derive(thiserror::Error, Debug)]
pub enum LoadModelError {
    #[error(transparent)]
    IoError(#[from] Error),

    #[error(transparent)]
    LoadError(#[from] tobj::LoadError),

    #[error(transparent)]
    LoadTextureError(#[from] LoadTextureError),
}

pub async fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> Result<model::Model, LoadModelError> {
    //! Loads the contents of an asset into a model object frmo the file system

    log::info!("Loading {:?}, as a Model", file_name);

    // Load the model file, using a Cursor to allow the tobj crate to read from any section
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    // Load the string into a tobj model, loading the mesh and materials
    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text = load_string(&p).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    // Iterate through the materials loading the diffuse and normal textures from the file system
    let mut materials = Vec::new();
    for material in obj_materials? {
        let diffuse_texture = load_texture(&material.diffuse_texture, device, queue).await?;
        let normal_texture = load_texture(&material.normal_texture, device, queue).await?;

        materials.push(model::Material::new(
            device,
            &material.name,
            diffuse_texture,
            normal_texture,
            layout,
        ))
    }

    // iterate through the meshes, converting them into the Mesh object
    // which contains information used for lighting such as a normal,
    // tangent and bitangent
    let meshes = models
        .into_iter()
        .map(|model| {
            // Map each vertex to a ModelVertex
            let mut vertices = (0..model.mesh.positions.len() / 3)
                .map(|i| model::ModelVertex {
                    position: [
                        model.mesh.positions[i * 3],
                        model.mesh.positions[i * 3 + 1],
                        model.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: [model.mesh.texcoords[i * 2], model.mesh.texcoords[i * 2 + 1]],
                    normal: [
                        model.mesh.normals[i * 3],
                        model.mesh.normals[i * 3 + 1],
                        model.mesh.normals[i * 3 + 2],
                    ],
                    tangent: [0.0; 3],
                    bitangent: [0.0; 3],
                })
                .collect::<Vec<_>>();

            // Get the vertices and indices of the model loaded by tobj
            let indices = &model.mesh.indices;
            let mut triangles_included = vec![0; vertices.len()];

            //
            for c in indices.chunks(3) {
                // Get the vertices specified at the point of each index
                let v0 = vertices[c[0] as usize];
                let v1 = vertices[c[1] as usize];
                let v2 = vertices[c[2] as usize];

                // Convert the vertices into vectors
                let pos0: cgmath::Vector3<_> = v0.position.into();
                let pos1: cgmath::Vector3<_> = v1.position.into();
                let pos2: cgmath::Vector3<_> = v2.position.into();

                let uv0: cgmath::Vector2<_> = v0.tex_coords.into();
                let uv1: cgmath::Vector2<_> = v1.tex_coords.into();
                let uv2: cgmath::Vector2<_> = v2.tex_coords.into();

                // Get the difference between the 0th position and 1st and 2nd position
                let delta_pos1 = pos1 - pos0;
                let delta_pos2 = pos2 - pos0;

                let delta_uv1 = uv1 - uv0;
                let delta_uv2 = uv2 - uv0;

                // Caluclate the tangent and bitangent of the face
                let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

                // Set the tangent and bitangent of each vertex
                vertices[c[0] as usize].tangent =
                    (tangent + cgmath::Vector3::from(vertices[c[0] as usize].tangent)).into();
                vertices[c[1] as usize].tangent =
                    (tangent + cgmath::Vector3::from(vertices[c[1] as usize].tangent)).into();
                vertices[c[2] as usize].tangent =
                    (tangent + cgmath::Vector3::from(vertices[c[2] as usize].tangent)).into();
                vertices[c[0] as usize].bitangent =
                    (bitangent + cgmath::Vector3::from(vertices[c[0] as usize].bitangent)).into();
                vertices[c[1] as usize].bitangent =
                    (bitangent + cgmath::Vector3::from(vertices[c[1] as usize].bitangent)).into();
                vertices[c[2] as usize].bitangent =
                    (bitangent + cgmath::Vector3::from(vertices[c[2] as usize].bitangent)).into();

                // Increase the count of each index used to average the tangent and bitangent
                triangles_included[c[0] as usize] += 1;
                triangles_included[c[1] as usize] += 1;
                triangles_included[c[2] as usize] += 1;
            }

            // average the tangent and bitangent vectors
            for (i, n) in triangles_included.into_iter().enumerate() {
                let denom = 1.0 / n as f32;
                let mut v = &mut vertices[i];
                v.tangent = (cgmath::Vector3::from(v.tangent) * denom).into();
                v.bitangent = (cgmath::Vector3::from(v.bitangent) * denom).into();
            }

            // Create a vertex buffer from the mesh
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            // Create an index buffer from the mesh
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_name)),
                contents: bytemuck::cast_slice(&model.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            // Create the mesh
            model::Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: model.mesh.indices.len() as u32,
                material: model.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    // Bundle the meshes and materials together into one object
    Ok(model::Model { meshes, materials })
}
