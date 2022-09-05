use std::collections::HashMap;

use cgmath::{EuclideanSpace, InnerSpace, Point3, Vector3};

use image::{DynamicImage, Rgb, Rgb32FImage, RgbImage, Rgba, Rgba32FImage, RgbaImage};
use itertools::Itertools;

use crate::renderer::{
    model::{Material, Mesh, Model, ModelVertex},
    texture::Texture,
};

pub struct Icosphere {
    vertices: Vec<Point3<f32>>,
    indices: Vec<usize>,
    radius: f32,
    detail_level: usize,
}
impl Icosphere {
    pub fn new(radius: f32, detail_level: usize) -> Self {
        let phi: f32 = 1.0 + 5.0f32.sqrt() / 2.0;

        let mut vectors = vec![
            (phi, 1.0, 0.0).into(),
            (phi, -1.0, 0.0).into(),
            (-phi, 1.0, 0.0).into(),
            (-phi, -1.0, 0.0).into(),
            (0.0, phi, 1.0).into(),
            (0.0, phi, -1.0).into(),
            (0.0, -phi, 1.0).into(),
            (0.0, -phi, -1.0).into(),
            (1.0, 0.0, phi).into(),
            (-1.0, 0.0, phi).into(),
            (1.0, 0.0, -phi).into(),
            (-1.0, 0.0, -phi).into(),
        ];

        #[rustfmt::skip]
        let mut indices = vec![
            9, 4, 8,
            9, 2, 4,
            2, 5, 4,
            4, 5, 0,
            4, 0, 8,
            0, 1, 8,
            0, 10, 1,
            5, 10, 0,
            5, 11, 10,
            11, 7, 10,
            7, 1, 10,
            7, 6, 1,
            7, 3, 6,
            3, 9, 6,
            9, 8, 6,
            6, 8, 1,
            2, 9, 3,
            2, 3, 11,
            2, 11, 5,
            7, 11, 3,
        ];

        for _ in 0..detail_level {
            Self::subdivide(&mut vectors, &mut indices);
        }

        let vertices = vectors
            .into_iter()
            .map(|v| Point3::from_vec(v.normalize() * radius))
            .collect();

        Self {
            vertices,
            indices,
            radius,
            detail_level,
        }
    }

    fn subdivide(vectors: &mut Vec<Vector3<f32>>, indices: &mut Vec<usize>) {
        let mut midpoint_indices = HashMap::new();

        let mut new_indices = Vec::with_capacity(indices.len() * 4);

        for (&i0, &i1, &i2) in indices.iter().tuple_windows().step_by(3) {
            let mid01 = Self::get_midpoint(vectors, &mut midpoint_indices, i0, i1);
            let mid02 = Self::get_midpoint(vectors, &mut midpoint_indices, i0, i2);
            let mid12 = Self::get_midpoint(vectors, &mut midpoint_indices, i1, i2);

            #[rustfmt::skip]
            new_indices.append(&mut vec![
                i0, mid01, mid02,
                i1, mid12, mid01,
                i2, mid02, mid12,
                mid02, mid01, mid12,
            ]);
        }

        *indices = new_indices;
    }

    fn get_midpoint(
        vectors: &mut Vec<Vector3<f32>>,
        midpoint_indices: &mut HashMap<(usize, usize), usize>,
        i0: usize,
        i1: usize,
    ) -> usize {
        let key = (i0.min(i1), i0.max(i1));

        if let Some(&index) = midpoint_indices.get(&key) {
            return index;
        }

        let v0 = vectors[i0];
        let v1 = vectors[i1];

        let mid = (v0 + v1) / 2.0;

        if let Some(index) = vectors.iter().position(|&v| v == mid) {
            index
        } else {
            let index = vectors.len();
            vectors.push(mid);
            midpoint_indices.insert(key, index);

            index
        }
    }

    pub fn into_model(
        self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: String,
        colour: [f32; 4],
        layout: &wgpu::BindGroupLayout,
    ) -> Model {
        let indices: Vec<_> = self.indices.into_iter().map(|i| i as u32).collect();
        let mut vertices: Vec<ModelVertex> = self
            .vertices
            .into_iter()
            .map(|v| ModelVertex {
                position: v.into(),
                tex_coords: {
                    let v = -v.to_vec().normalize();
                    [
                        0.5 + f32::atan2(v.x, v.y) / std::f32::consts::TAU,
                        0.5 + v.y.asin() / std::f32::consts::PI,
                    ]
                },
                normal: v.to_vec().normalize().into(),
                tangent: [0.0; 3],
                bitangent: [0.0; 3],
            })
            .collect();

        let mut triangles_included = vec![0; vertices.len()];

        for c in indices.chunks(3) {
            let v0 = vertices[c[0] as usize];
            let v1 = vertices[c[1] as usize];
            let v2 = vertices[c[2] as usize];

            let pos0: cgmath::Vector3<_> = v0.position.into();
            let pos1: cgmath::Vector3<_> = v1.position.into();
            let pos2: cgmath::Vector3<_> = v2.position.into();

            let uv0: cgmath::Vector2<_> = v0.tex_coords.into();
            let uv1: cgmath::Vector2<_> = v1.tex_coords.into();
            let uv2: cgmath::Vector2<_> = v2.tex_coords.into();

            let delta_pos1 = pos1 - pos0;
            let delta_pos2 = pos2 - pos0;

            let delta_uv1 = uv1 - uv0;
            let delta_uv2 = uv2 - uv0;

            let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
            let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
            let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

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

            triangles_included[c[0] as usize] += 1;
            triangles_included[c[1] as usize] += 1;
            triangles_included[c[2] as usize] += 1;
        }

        for (i, n) in triangles_included.into_iter().enumerate() {
            let denom = 1.0 / n as f32;
            let mut v = &mut vertices[i];
            v.tangent = (cgmath::Vector3::from(v.tangent) * denom).into();
            v.bitangent = (cgmath::Vector3::from(v.bitangent) * denom).into();
        }

        let mut texture = Rgba32FImage::new(100, 100);
        texture.pixels_mut().for_each(|p| *p = Rgba(colour));

        let texture = Texture::from_image(
            device,
            queue,
            &DynamicImage::ImageRgba32F(texture),
            Some(&format!("{:?} Texture", name)),
        );

        let mut normal = Rgb32FImage::new(10, 10);
        normal.pixels_mut().for_each(|p| *p = Rgb([1.0, 1.0, 1.0]));

        let normal = Texture::from_image(
            device,
            queue,
            &DynamicImage::ImageRgb32F(normal),
            Some(&format!("{:?} Normal Texture", name)),
        );

        let meshes = vec![Mesh::new(device, name.clone(), vertices, indices, 0)];
        let materials = vec![Material::new(
            device,
            &format!("{:?} Material", name),
            texture,
            normal,
            layout,
        )];

        Model { meshes, materials }
    }
}
