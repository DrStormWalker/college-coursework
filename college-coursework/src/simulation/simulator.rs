use log::debug;
use rayon::prelude::*;
use specs::{Entities, Join, ParJoin, Read, ReadStorage, System, WriteStorage};

use crate::util::{Vec3, BIG_G};

use super::{components::DeltaTime, Mass, Position, Velocity};

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
        Entities<'a>,
    );

    fn run(&mut self, (mut positions, mut velocities, mass, dt, entities): Self::SystemData) {
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
                        let r = other.0 - pos.0;

                        // Apply Newton's equation for universal gravitation
                        // The equation has been manipulated
                        // F = m1 * a
                        // F = G * m1 * m2 / |r|^2
                        // m1 * a = G * m1 * m2 / |r|^2
                        // a = G * m2 / |r|^2
                        let a = BIG_G * mass.0 / r.magnitude_squared();

                        // Get the direction of the other body from this
                        // And project the acceleration into that direction
                        a * r.normalize()
                    })
                    .reduce(|a, b| a + b);

                // Apply the resultant acceleration to the velocity
                if let Some(resultant) = resultant {
                    vel.0 += resultant * dt.0.as_secs_f64();
                }
            });

        // Apply the velocity to the position
        (&mut positions, &velocities)
            .par_join()
            .for_each(|(mut pos, vel)| {
                pos.0 += vel.0 * dt.0.as_secs_f64();
            });
    }
}
