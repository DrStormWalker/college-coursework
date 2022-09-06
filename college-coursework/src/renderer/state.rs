use std::{rc::Rc, sync::Arc};

use cgmath::{Euler, InnerSpace, Rotation3, Zero};
use instant::Duration;
use specs::{Join, ReadStorage, World, Write};
use wgpu::{include_wgsl, util::DeviceExt};
use winit::{
    event::{ElementState, KeyboardInput, MouseButton, WindowEvent},
    window::Window,
};

use crate::{
    assets, models,
    renderer::{instance::InstanceRaw, light::LightUniform, vertex::Vertex},
    setup::Dispatchers,
    simulation::DeltaTime,
};

use super::{
    camera,
    components::RenderModel,
    instance,
    light::DrawLight,
    model::{self, DrawModel, Model},
    texture,
};

const NUM_INSTANCES_PER_ROW: u32 = 1;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

pub struct RenderPassContainer<'a>(wgpu::RenderPass<'a>);

pub struct State {
    surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: Arc<wgpu::Queue>,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,

    render_pipeline: wgpu::RenderPipeline,
    light_render_pipeline: wgpu::RenderPipeline,

    /*vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_vertices: u32,*/
    diffuse_texture: texture::Texture,
    //diffuse_bind_group: wgpu::BindGroup,
    camera: camera::Camera,
    camera_projection: camera::Projection,
    camera_uniform: camera::CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    light_bind_group: wgpu::BindGroup,
    pub camera_controller: Box<dyn camera::CameraController>,

    depth_texture: texture::Texture,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
}
impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),

                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let diffuse_bytes = include_bytes!("assets/happy-tree.png");
        let diffuse_texture =
            texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        /*let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });*/

        let light_uniform = LightUniform::new([0.0, 4.0, 0.0], [1.0, 1.0, 1.0]);

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let camera = camera::Camera::new(
            (0.0, 5.0, 10.0),
            Euler {
                x: cgmath::Deg(-20.0),
                y: cgmath::Deg(-90.0),
                z: cgmath::Deg(0.0),
            },
        );
        let camera_projection =
            camera::Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 4000.0);

        let camera_controller = Box::new(camera::FreeCameraController::new(20.0, 40.0, 1.0, 1.0));

        let mut camera_uniform = camera::CameraUniform::new();

        camera_uniform.update_view_proj(&camera, &camera_projection);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let shader = include_wgsl!("shaders/shader.wgsl");

        let render_pipeline = Self::create_render_pipeline(
            &device,
            &render_pipeline_layout,
            config.format,
            Some(texture::Texture::DEPTH_FORMAT),
            &[model::ModelVertex::desc(), instance::InstanceRaw::desc()],
            shader,
        );

        let light_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader = include_wgsl!("shaders/light.wgsl");

        let light_render_pipeline = Self::create_render_pipeline(
            &device,
            &light_render_pipeline_layout,
            config.format,
            Some(texture::Texture::DEPTH_FORMAT),
            &[model::ModelVertex::desc()],
            shader,
        );

        /*let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_vertices = INDICES.len() as u32;

        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position = cgmath::Vector3 {
                        x: x as f32,
                        y: 0.0,
                        z: z as f32,
                    } - INSTANCE_DISPLACEMENT;

                    let rotation = if position.is_zero() {
                        // this is needed so an object at (0, 0, 0) won't get scaled to zero
                        // as Quaternions can effect scale if they're not created correctly
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                    };

                    instance::Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();*/

        /*let obj_model = assets::load_model("cube.obj", &device, &queue, &texture_bind_group_layout)
            .await
            .unwrap();

        let obj_model = models::sphere::Icosphere::new(1.0, 2).into_model(
            &device,
            &queue,
            "Icosphere".to_string(),
            [0.0, 1.0, 0.0, 1.0],
            &texture_bind_group_layout,
        );

        const SPACE_BETWEEN: f32 = 3.0;
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                    let position = cgmath::Vector3 { x, y: 0.0, z };

                    let rotation = if position.is_zero() {
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                    };

                    instance::Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let instances = vec![instance::Instance {
            position: [0.0; 3].into(),
            rotation: cgmath::Quaternion::zero(),
        }];

        let instance_data = instances
            .iter()
            .map(instance::Instance::to_raw)
            .collect::<Vec<_>>();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });*/

        Self {
            surface,
            device,
            queue: Arc::new(queue),
            config,
            size,
            render_pipeline,
            light_render_pipeline,
            /*vertex_buffer,
            index_buffer,
            num_vertices,*/
            diffuse_texture,
            //diffuse_bind_group,
            camera,
            camera_projection,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            light_bind_group,
            camera_controller,
            depth_texture,
            texture_bind_group_layout,
        }
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        colour_format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
        vertex_layouts: &[wgpu::VertexBufferLayout],
        shader: wgpu::ShaderModuleDescriptor,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(shader);

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: vertex_layouts,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: colour_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");

            self.camera_projection
                .resize(new_size.width, new_size.height);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self, dt: Duration, world: &mut World, dispatchers: &mut Dispatchers) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform
            .update_view_proj(&self.camera, &self.camera_projection);

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        world.exec(|(mut delta,): (Write<DeltaTime>,)| {
            delta.0 = dt;
        });

        dispatchers.simulation_dispatcher.dispatch(world);
    }

    pub fn render(&mut self, world: &mut World) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        world.exec(|(models,): (ReadStorage<RenderModel>,)| {
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                });

                //render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

                /*#[cfg(feature = "light")]
                render_pass.set_pipeline(&self.light_render_pipeline);
                #[cfg(feature = "light")]
                render_pass.draw_light_model(
                    &self.obj_model,
                    &self.camera_bind_group,
                    &self.light_bind_group,
                );

                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.draw_model_instanced(
                    &self.obj_model,
                    0..self.instances.len() as u32,
                    &self.camera_bind_group,
                    &self.light_bind_group,
                );*/

                //render_pass.draw_indexed(0..self.num_vertices, 0, 0..self.instances.len() as _);

                render_pass.set_pipeline(&self.render_pipeline);

                (&models).join().for_each(|model| {
                    render_pass.set_vertex_buffer(1, model.instance_buffer.slice(..));
                    render_pass.draw_model(
                        &model.model,
                        &self.camera_bind_group,
                        &self.light_bind_group,
                    );
                })
            }

            self.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        });

        Ok(())
    }
}
