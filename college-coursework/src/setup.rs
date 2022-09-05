use std::sync::Arc;

use cgmath::{Quaternion, Vector3, Zero};
use error_stack::Result;
use specs::{Builder, Dispatcher, DispatcherBuilder, World, WorldExt};
use thiserror::Error;

use crate::{
    models::sphere::Icosphere,
    renderer::{components::RenderModel, instance::Instance},
    simulation::{
        self, BodyType, Identifier, InstanceUpdater, InteractionFlags, InteractionHandler, Mass,
        Position, Simulator, TimeScale, Velocity, SUN,
    },
};

#[derive(Debug, Error)]
pub enum SetupError {}

pub struct Dispatchers<'a, 'b> {
    pub simulation_dispatcher: Dispatcher<'a, 'b>,
}

pub async fn setup<'a, 'b>(
    device: &wgpu::Device,
    queue: Arc<wgpu::Queue>,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
) -> Result<(World, Dispatchers<'a, 'b>), SetupError> {
    let mut world = World::new();

    world.register::<Identifier>();
    world.register::<Position>();
    world.register::<Velocity>();
    world.register::<Mass>();
    world.register::<RenderModel>();
    world.register::<InteractionHandler>();

    world
        .create_entity()
        .with(SUN.get_identifier())
        .with(SUN.get_pos())
        .with(SUN.get_vel())
        .with(SUN.get_mass())
        .with(RenderModel::new(
            device,
            Icosphere::new(8.0, 4).into_model(
                device,
                &queue,
                "The Sun".into(),
                [252.0 / 255.0, 229.0 / 255.0, 112.0 / 255.0, 0.0],
                texture_bind_group_layout,
            ),
            Instance::new([0.0; 3].into(), Quaternion::zero()),
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            Some("The Sun"),
        ))
        .with(InteractionHandler::new(
            InteractionFlags::STAR,
            BodyType::Star,
        ))
        .build();

    for planet in simulation::planets() {
        world
            .create_entity()
            .with(planet.get_identifier())
            .with(planet.get_pos())
            .with(planet.get_vel())
            .with(planet.get_mass())
            .with(RenderModel::new(
                device,
                Icosphere::new(2.5, 3).into_model(
                    device,
                    &queue,
                    planet.get_identifier().get_id().to_string(),
                    [0.0 / 255.0, 0.0 / 255.0, 255.0 / 255.0, 1.0],
                    texture_bind_group_layout,
                ),
                Instance::new(planet.get_pos().0 / 4_000_000_000.0, Quaternion::zero()),
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                Some(planet.get_identifier().get_id()),
            ))
            .with(InteractionHandler::new(
                InteractionFlags::all(),
                BodyType::Planet,
            ))
            .build();
    }

    world.insert(queue);
    world.insert(TimeScale::new(315576000.0, 20));

    let simulation_dispatcher = DispatcherBuilder::new()
        .with(Simulator::new(), "sys_simulator", &[])
        .with(
            InstanceUpdater::new(),
            "sys_instance_updater",
            &["sys_simulator"],
        )
        .build();

    Ok((
        world,
        Dispatchers {
            simulation_dispatcher,
        },
    ))
}
