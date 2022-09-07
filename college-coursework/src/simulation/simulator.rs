use std::sync::Arc;

use cgmath::{InnerSpace, Quaternion, Zero};
use crossbeam::channel::Receiver;
use fltk::app;
use log::debug;
use rayon::prelude::*;
use specs::{Entities, Join, ParJoin, Read, ReadExpect, ReadStorage, System, Write, WriteStorage};

use crate::{
    panel::{BodyState, GlobalState, UiMessage, VectorStateChange},
    renderer::{
        camera::{CameraPosition, CameraSpeed},
        components::RenderModel,
    },
    util::BIG_G,
};

use super::{
    components::{DeltaTime, TimeScale},
    GravitationalConstant, Identifier, InteractionFlags, InteractionHandler, Mass, Position,
    PositionScaleFactor, Velocity,
};

pub struct Simulator;
impl Simulator {
    pub fn new() -> Self {
        Self {}
    }
}
impl<'a> System<'a> for Simulator {
    type SystemData = (
        WriteStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        ReadStorage<'a, Mass>,
        ReadStorage<'a, InteractionHandler>,
        Read<'a, DeltaTime>,
        Read<'a, TimeScale>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (mut positions, mut velocities, mass, interaction_handlers, dt, time_scale, entities): Self::SystemData,
    ) {
        for _ in 0..time_scale.iterations {
            // Iterate over every entity in parallel
            (
                &entities,
                &positions,
                &mut velocities,
                &interaction_handlers,
            )
                .par_join()
                .for_each(|(e, pos, mut vel, interaction_handler)| {
                    // Get a resultant acceleration using iterators
                    let resultant = (&entities, &positions, &mass, &interaction_handlers)
                        .join()
                        // Make sure the body does not try to interact with itself
                        .filter(|(o, _pos, _mass, _interaction_handler)| e.id() != o.id())
                        .filter(|(_, _pos, _mass, other_interaction_handler)| {
                            let other_flags: InteractionFlags =
                                other_interaction_handler.body_type.into();
                            interaction_handler.flags & other_flags == other_flags
                        })
                        .map(|(_, other, mass, _interaction_handler)| {
                            // Displacement from one body to the other
                            let r = other.0 - pos.0;

                            // Apply Newton's equation for universal gravitation
                            // The equation has been manipulated
                            // F = m1 * a
                            // F = G * m1 * m2 / |r|^2
                            // m1 * a = G * m1 * m2 / |r|^2
                            // a = G * m2 / |r|^2
                            let a = BIG_G * mass.0 / r.magnitude2();

                            // Get the direction of the other body from this
                            // And project the acceleration into that direction
                            a * r.normalize()
                        })
                        .reduce(|a, b| a + b);

                    // Apply the resultant acceleration to the velocity
                    if let Some(resultant) = resultant {
                        vel.0 += resultant * time_scale.time_scale * dt.0.as_secs_f64();
                    }
                });

            // Apply the velocity to the position
            (&mut positions, &velocities)
                .par_join()
                .for_each(|(mut pos, vel)| {
                    pos.0 += vel.0 * time_scale.time_scale * dt.0.as_secs_f64();
                });
        }
    }
}

pub struct InstanceUpdater;
impl InstanceUpdater {
    pub fn new() -> Self {
        Self {}
    }
}
impl<'a> System<'a> for InstanceUpdater {
    type SystemData = (
        ReadStorage<'a, Position>,
        WriteStorage<'a, RenderModel>,
        ReadExpect<'a, Arc<wgpu::Queue>>,
    );

    fn run(&mut self, (positions, mut models, queue): Self::SystemData) {
        (&positions, &mut models)
            .join()
            .for_each(|(position, model)| {
                model.update_instance(
                    &queue,
                    position.0.map(|a| a as f32) / 4_000_000_000.0,
                    Quaternion::zero(),
                );
            });
    }
}

pub struct UiUpdater {
    sender: app::Sender<UiMessage>,
}
impl UiUpdater {
    pub fn new(sender: app::Sender<UiMessage>) -> Self {
        Self { sender }
    }
}
impl<'a> System<'a> for UiUpdater {
    type SystemData = (
        ReadStorage<'a, Identifier>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Velocity>,
        Read<'a, CameraSpeed>,
        Read<'a, CameraPosition>,
    );

    fn run(
        &mut self,
        (identifiers, positions, velocities, camera_speed, camera_position): Self::SystemData,
    ) {
        (&identifiers, &positions, &velocities)
            .join()
            .for_each(|(id, position, velocity)| {
                self.sender.send(UiMessage::BodyState {
                    id: id.get_id().to_string(),
                    state: BodyState::ChangePosition(VectorStateChange::X(position.0.x)),
                });

                self.sender.send(UiMessage::BodyState {
                    id: id.get_id().to_string(),
                    state: BodyState::ChangePosition(VectorStateChange::Y(position.0.y)),
                });

                self.sender.send(UiMessage::BodyState {
                    id: id.get_id().to_string(),
                    state: BodyState::ChangePosition(VectorStateChange::Z(position.0.z)),
                });

                self.sender.send(UiMessage::BodyState {
                    id: id.get_id().to_string(),
                    state: BodyState::ChangeVelocity(VectorStateChange::X(velocity.0.x)),
                });

                self.sender.send(UiMessage::BodyState {
                    id: id.get_id().to_string(),
                    state: BodyState::ChangeVelocity(VectorStateChange::Y(velocity.0.y)),
                });

                self.sender.send(UiMessage::BodyState {
                    id: id.get_id().to_string(),
                    state: BodyState::ChangeVelocity(VectorStateChange::Z(velocity.0.z)),
                });
            });

        self.sender
            .send(UiMessage::GlobalState(GlobalState::ChangeCameraPosition(
                VectorStateChange::X(camera_position.0.x as f64),
            )));

        self.sender
            .send(UiMessage::GlobalState(GlobalState::ChangeCameraPosition(
                VectorStateChange::Y(camera_position.0.y as f64),
            )));

        self.sender
            .send(UiMessage::GlobalState(GlobalState::ChangeCameraPosition(
                VectorStateChange::Z(camera_position.0.z as f64),
            )));

        self.sender
            .send(UiMessage::GlobalState(GlobalState::ChangeCameraSpeed(
                camera_speed.0 as f64,
            )));
    }
}

pub struct ApplicationUpdater {
    receiver: Receiver<UiMessage>,
}
impl ApplicationUpdater {
    pub fn new(receiver: Receiver<UiMessage>) -> Self {
        Self { receiver }
    }
}
impl<'a> System<'a> for ApplicationUpdater {
    type SystemData = (
        ReadStorage<'a, Identifier>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        WriteStorage<'a, Mass>,
        Write<'a, GravitationalConstant>,
        Write<'a, PositionScaleFactor>,
        Write<'a, CameraPosition>,
        Write<'a, CameraSpeed>,
    );

    fn run(
        &mut self,
        (
            identifiers,
            positions,
            velocities,
            mut masses,
            mut gravitational_constant,
            mut scale_factor,
            mut camera_position,
            mut camera_speed,
        ): Self::SystemData,
    ) {
        if let Ok(msg) = self.receiver.try_recv() {
            match msg {
                UiMessage::BodyState { id, state } => match state {
                    BodyState::ChangeMass(new_mass) => (&identifiers, &mut masses)
                        .join()
                        .filter(|(identifier, _mass)| identifier.get_id() == &id)
                        .for_each(|(_id, mass)| mass.0 = new_mass),
                    _ => {}
                },
                UiMessage::GlobalState(state) => match state {
                    GlobalState::ChangeCameraPosition(component) => match component {
                        VectorStateChange::X(x) => camera_position.0.x = x as f32,
                        VectorStateChange::Y(y) => camera_position.0.y = y as f32,
                        VectorStateChange::Z(z) => camera_position.0.z = z as f32,
                    },
                    GlobalState::ChangeCameraSpeed(speed) => camera_speed.0 = speed as f32,
                    GlobalState::ChangeGravitationalConstant(constant) => {
                        gravitational_constant.0 = constant
                    }
                    GlobalState::ChangeScale(scale) => scale_factor.0 = scale,
                    _ => {}
                },
            }
        }
    }
}
