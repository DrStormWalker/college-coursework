use cgmath::{Quaternion, Zero};
use error_stack::Result;
use specs::{Builder, Dispatcher, DispatcherBuilder, World, WorldExt};
use thiserror::Error;

use crate::{
    models::sphere::Icosphere,
    renderer::{components::RenderModel, instance::Instance},
    simulation::{
        self, BodyType, Identifier, InteractionFlags, InteractionHandler, Mass, Position,
        Simulator, Velocity, SUN,
    },
};

#[derive(Debug, Error)]
pub enum SetupError {}

pub struct Dispatchers<'a, 'b> {
    pub simulation_dispatcher: Dispatcher<'a, 'b>,
}

pub async fn setup<'a, 'b>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
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
            Icosphere::new(5.0, 4).into_model(
                device,
                queue,
                "The Sun".into(),
                [252.0 / 255.0, 229.0 / 255.0, 112.0 / 255.0, 0.0],
                texture_bind_group_layout,
            ),
            Instance::new([0.0; 3].into(), Quaternion::zero()),
            wgpu::BufferUsages::VERTEX,
            Some("The Sun"),
        ))
        .with(InteractionHandler::new(
            InteractionFlags::STAR,
            BodyType::Star,
        ))
        .build();

    /*for planet in simulation::planets() {
        world
            .create_entity()
            .with(planet.get_identifier())
            .with(planet.get_pos())
            .with(planet.get_vel())
            .with(planet.get_mass())
            .with(InteractionHandler::new(
                InteractionFlags::all(),
                BodyType::Planet,
            ))
            .build();
    }*/

    let simulation_dispatcher = DispatcherBuilder::new()
        .with(Simulator::new(), "sys_simulator", &[])
        .build();

    Ok((
        world,
        Dispatchers {
            simulation_dispatcher,
        },
    ))
}
