use std::sync::Arc;

use cgmath::{InnerSpace, Quaternion, Zero};
use log::debug;
use rayon::prelude::*;
use specs::{Entities, Join, ParJoin, Read, ReadExpect, ReadStorage, System, WriteStorage};

use crate::{renderer::components::RenderModel, util::BIG_G};

use super::{
    components::{DeltaTime, TimeScale},
    Mass, Position, Velocity,
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
        Read<'a, DeltaTime>,
        Read<'a, TimeScale>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (mut positions, mut velocities, mass, dt, time_scale, entities): Self::SystemData,
    ) {
        for _ in 0..time_scale.iterations {
            // Iterate over every entity in parallel
            (&entities, &positions, &mut velocities)
                .par_join()
                .for_each(|(e, pos, mut vel)| {
                    // Get a resultant acceleration using iterators
                    let resultant = (&entities, &positions, &mass)
                        .join()
                        // Make sure the body does not try to interact with itself
                        .filter(|(o, _pos, _mass)| e.id() != o.id())
                        .map(|(o, other, mass)| {
                            // Displacement from one body to the other
                            let r = pos.0 - other.0;

                            // Apply Newton's equation for universal gravitation
                            // The equation has been manipulated
                            // F = m1 * a
                            // F = G * m1 * m2 / |r|^2
                            // m1 * a = G * m1 * m2 / |r|^2
                            // a = G * m2 / |r|^2
                            let a = -1.0 * BIG_G as f32 * mass.0 / r.magnitude2();

                            // Get the direction of the other body from this
                            // And project the acceleration into that direction
                            a * r.normalize()
                        })
                        .reduce(|a, b| a + b);

                    // Apply the resultant acceleration to the velocity
                    if let Some(resultant) = resultant {
                        vel.0 += resultant * time_scale.time_scale * dt.0.as_secs_f32();
                    }
                });

            // Apply the velocity to the position
            (&mut positions, &velocities)
                .par_join()
                .for_each(|(mut pos, vel)| {
                    pos.0 += vel.0 * time_scale.time_scale * dt.0.as_secs_f32();
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
                model.update_instance(&queue, position.0 / 4_000_000_000.0, Quaternion::zero());
            });
    }
}
