use std::sync::Arc;

use cgmath::{Quaternion, Vector3, Zero};
use crossbeam::channel::Receiver;
use error_stack::Result;
use fltk::app;
use specs::{
    Builder, Dispatcher, DispatcherBuilder, Join, Read, ReadExpect, ReadStorage, World, WorldExt,
};
use thiserror::Error;

use crate::{
    models::sphere::Icosphere,
    panel::{BodyState, GlobalState, UiMessage},
    renderer::{
        components::{CameraCenter, RenderModel, UpdateCameraDisplacement, UpdateCameraPosition},
        instance::Instance,
    },
    simulation::{
        self, ApplicationUpdater, BodyType, GravitationalConstant, Identifier, InstanceUpdater,
        InteractionFlags, InteractionHandler, Mass, Position, PositionScaleFactor, Simulator,
        TimeScale, UiUpdater, Velocity, SUN,
    },
    util::BIG_G,
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
    sender: app::Sender<UiMessage>,
    receiver: Receiver<UiMessage>,
) -> Result<(World, Dispatchers<'a, 'b>), SetupError> {
    //! Setup the Enityt Component System
    let mut world = World::new();

    // Register the components
    world.register::<Identifier>();
    world.register::<Position>();
    world.register::<Velocity>();
    world.register::<Mass>();
    world.register::<RenderModel>();
    world.register::<InteractionHandler>();

    // Create the Sun entity
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
                SUN.get_colour(),
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

    // Create the planets
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
                    planet.get_colour(),
                    texture_bind_group_layout,
                ),
                Instance::new(
                    planet.get_pos().0.map(|a| a as f32) / 4_000_000_000.0,
                    Quaternion::zero(),
                ),
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                Some(planet.get_identifier().get_id()),
            ))
            .with(InteractionHandler::new(
                InteractionFlags::all(),
                BodyType::Planet,
            ))
            .build();
    }

    // Add the global states to thje Entity Component System
    world.insert(queue);
    world.insert(TimeScale::new(3155760.0, 20));
    world.insert(GravitationalConstant(BIG_G));
    world.insert(PositionScaleFactor(4_000_000_000.0));
    world.insert(CameraCenter::new(SUN.get_identifier()));

    // Update all of the global states in the Ui
    world.exec(
        |(identifiers, masses, time_scale, constant, scale_factor): (
            ReadStorage<Identifier>,
            ReadStorage<Mass>,
            ReadExpect<TimeScale>,
            Read<GravitationalConstant>,
            Read<PositionScaleFactor>,
        )| {
            sender.send(UiMessage::GlobalState(GlobalState::ChangeScale(
                time_scale.total_time_elapsed,
            )));

            sender.send(UiMessage::GlobalState(
                GlobalState::ChangeGravitationalConstant(constant.0),
            ));

            sender.send(UiMessage::GlobalState(GlobalState::ChangeScale(
                scale_factor.0,
            )));

            (&identifiers, &masses).join().for_each(|(id, mass)| {
                sender.send(UiMessage::BodyState {
                    id: id.get_id().to_string(),
                    state: BodyState::ChangeMass(mass.0),
                });
            });
        },
    );

    // Register the systems
    let simulation_dispatcher = DispatcherBuilder::new()
        .with(
            UpdateCameraDisplacement {},
            "sys_update_camera_displacement",
            &[],
        )
        .with(
            Simulator::new(),
            "sys_simulator",
            &["sys_update_camera_displacement"],
        )
        .with(
            InstanceUpdater::new(),
            "sys_instance_updater",
            &["sys_simulator"],
        )
        .with(UpdateCameraPosition {}, "sys_update_camera_position", &[])
        .with(
            ApplicationUpdater::new(receiver),
            "sys_app_updater",
            &["sys_simulator", "sys_update_camera_position"],
        )
        .with(
            UiUpdater::new(sender),
            "sys_ui_updater",
            &["sys_simulator", "sys_update_camera_position"],
        )
        .build();

    Ok((
        world,
        Dispatchers {
            simulation_dispatcher,
        },
    ))
}
