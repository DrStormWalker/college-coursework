use cgmath::{EuclideanSpace, Point3, Quaternion, Vector3, Zero};
use specs::{
    Component, Join, Read, ReadExpect, ReadStorage, System, VecStorage, Write, WriteExpect,
};
use wgpu::util::DeviceExt;

use crate::simulation::{Identifier, Position, PositionScaleFactor};

use super::{camera::CameraPosition, instance::Instance, model::Model};

/// Represents a model in the Entity COmponent System
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
        //! Create a new render model

        let buffer_label = label.map(|label| format!("{:?} Instance Buffer", label));

        // Create a new buffer for the instance
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
        //! Updates the position of the model for use by the GPU

        self.instance.position = position;
        self.instance.rotation = rotation;

        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&[self.instance.to_raw()]),
        );
    }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct CameraCenter {
    body: Identifier,
    displacement: Vector3<f32>,
}
impl CameraCenter {
    pub fn new(body: Identifier) -> Self {
        Self {
            body,
            displacement: Vector3::<f32>::zero(),
        }
    }
}

pub struct UpdateCameraDisplacement;
impl<'a> System<'a> for UpdateCameraDisplacement {
    type SystemData = (
        ReadStorage<'a, Identifier>,
        ReadStorage<'a, Position>,
        Read<'a, PositionScaleFactor>,
        Read<'a, CameraPosition>,
        WriteExpect<'a, CameraCenter>,
    );

    fn run(
        &mut self,
        (identifiers, positions, scale, camera_position, mut camera_center): Self::SystemData,
    ) {
        if let Some((_, position)) = (&identifiers, &positions)
            .join()
            .filter(|(id, _)| id.get_id() == camera_center.body.get_id())
            .next()
        {
            camera_center.displacement = camera_position.0.to_vec()
                - Vector3::<f32>::from((
                    position.0.x as f32,
                    position.0.y as f32,
                    position.0.z as f32,
                )) / scale.0 as f32;
        }
    }
}

pub struct UpdateCameraPosition;
impl<'a> System<'a> for UpdateCameraPosition {
    type SystemData = (
        ReadStorage<'a, Identifier>,
        ReadStorage<'a, Position>,
        Read<'a, PositionScaleFactor>,
        ReadExpect<'a, CameraCenter>,
        Write<'a, CameraPosition>,
    );

    fn run(
        &mut self,
        (identifiers, positions, scale, camera_center, mut camera_position): Self::SystemData,
    ) {
        if let Some((_, position)) = (&identifiers, &positions)
            .join()
            .filter(|(id, _)| id.get_id() == camera_center.body.get_id())
            .next()
        {
            camera_position.0 = Point3::<f32>::from_vec(
                Vector3::<f32>::from((
                    position.0.x as f32,
                    position.0.y as f32,
                    position.0.z as f32,
                )) / scale.0 as f32
                    + camera_center.displacement,
            );
        }
    }
}
