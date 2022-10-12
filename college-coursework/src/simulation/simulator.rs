use std::sync::Arc;

use cgmath::{InnerSpace, Quaternion, Zero};
use crossbeam::channel::Receiver;
use log::debug;
use rayon::prelude::*;
use specs::{Entities, Join, ParJoin, Read, ReadExpect, ReadStorage, System, Write, WriteStorage};

use crate::{
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
        Read<'a, GravitationalConstant>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (
            mut positions,
            mut velocities,
            mass,
            interaction_handlers,
            dt,
            time_scale,
            gravitational_constant,
            entities,
        ): Self::SystemData,
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
                        // Stop different types of bodys interacting if it will have negligable effect
                        // e.g. (planet effecting the sun)
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
                            let a = gravitational_constant.0 * mass.0 / r.magnitude2();

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
        Read<'a, PositionScaleFactor>,
        ReadExpect<'a, Arc<wgpu::Queue>>,
    );

    fn run(&mut self, (positions, mut models, scale_factor, queue): Self::SystemData) {
        (&positions, &mut models)
            .join()
            .for_each(|(position, model)| {
                model.update_instance(
                    &queue,
                    position.0.map(|a| a as f32) / scale_factor.0 as f32,
                    Quaternion::zero(),
                );
            });
    }
}
