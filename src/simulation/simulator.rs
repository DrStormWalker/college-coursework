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
        (&entities, &positions, &mut velocities)
            .par_join()
            .for_each(|(e, pos, mut vel)| {
                let resultant = (&entities, &positions, &mass)
                    .join()
                    .filter(|(o, _pos, _mass)| e.id() != o.id())
                    .map(|(o, other, mass)| {
                        let r = pos.0 - other.0;

                        let a = -1.0 * BIG_G * mass.0 / r.magnitude_squared();

                        a * r.normalize()
                    })
                    .reduce(|a, b| a + b);

                if let Some(resultant) = resultant {
                    vel.0 += resultant * dt.0;
                }
            });

        (&mut positions, &velocities)
            .par_join()
            .for_each(|(mut pos, vel)| {
                pos.0 += vel.0 * dt.0;
            });
    }
}
