use cgmath::{Quaternion, Vector3};
use specs::{Component, VecStorage};
use wgpu::util::DeviceExt;

use super::{instance::Instance, model::Model};

#[derive(Component)]
#[storage(VecStorage)]
pub struct RenderModel {
    pub model: Model,
    pub instance: Instance,
    pub instance_buffer: wgpu::Buffer,
}
impl RenderModel {
    pub fn new(
        device: &wgpu::Device,
        model: Model,
        instance: Instance,
        usage: wgpu::BufferUsages,
        label: Option<&str>,
    ) -> Self {
        let buffer_label = label.map(|label| format!("{:?} Instance Buffer", label));

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: buffer_label.as_ref().map(|label| label.as_str()),
            contents: bytemuck::cast_slice(&[instance.to_raw()]),
            usage,
        });

        Self {
            model,
            instance,
            instance_buffer,
        }
    }

    pub fn update_instance(
        &mut self,
        queue: &wgpu::Queue,
        position: Vector3<f32>,
        rotation: Quaternion<f32>,
    ) {
        self.instance.position = position;
        self.instance.rotation = rotation;

        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&[self.instance.to_raw()]),
        );
    }
}
