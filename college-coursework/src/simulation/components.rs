use cgmath::Vector3;
use instant::Duration;
use log::{debug, info};
use specs::{Component, Join, Read, ReadStorage, System, VecStorage};

use crate::renderer::camera::{CameraPosition, CameraSpeed};

// The position of an entity
#[derive(Debug, Clone, Copy)]
pub struct Position(pub Vector3<f64>);
impl From<Vector3<f64>> for Position {
    fn from(v: Vector3<f64>) -> Self {
        Self(v)
    }
}
impl Component for Position {
    type Storage = VecStorage<Self>;
}

// The velocity of an entity
#[derive(Debug, Clone, Copy)]
pub struct Velocity(pub Vector3<f64>);
impl From<Vector3<f64>> for Velocity {
    fn from(v: Vector3<f64>) -> Self {
        Self(v)
    }
}
impl Component for Velocity {
    type Storage = VecStorage<Self>;
}

// The mass of an entity
#[derive(Debug, Clone, Copy)]
pub struct Mass(pub f64);
impl From<f64> for Mass {
    fn from(m: f64) -> Self {
        Self(m)
    }
}
impl Component for Mass {
    type Storage = VecStorage<Self>;
}

// The Identifier and name of an entity
#[derive(Debug, Clone)]
pub struct Identifier {
    id: String,
    name: String,
}
impl Identifier {
    pub fn new(id: String, name: String) -> Self {
        Self { id, name }
    }

    pub fn get_id(&self) -> &str {
        self.id.as_str()
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }
}
impl Component for Identifier {
    type Storage = VecStorage<Self>;
}

// The delta time container struct
#[derive(Default, Copy, Clone)]
pub struct DeltaTime(pub Duration);

#[derive(Default, Copy, Clone)]
pub struct TimeScale {
    pub time_scale: f64,
    pub total_time_elapsed: f64,
    pub iterations: usize,
}
impl TimeScale {
    pub fn new(total_time_elapsed: f64, iterations: usize) -> Self {
        if iterations < 1 {
            panic!("Iterations cannot be less than 1");
        }

        Self {
            time_scale: total_time_elapsed / iterations as f64,
            total_time_elapsed,
            iterations,
        }
    }
}

#[derive(Default, Copy, Clone)]
pub struct GravitationalConstant(pub f64);

#[derive(Default, Copy, Clone)]
pub struct PositionScaleFactor(pub f64);

pub struct Printer;
impl Printer {
    pub fn new() -> Self {
        Self {}
    }
}
impl<'a> System<'a> for Printer {
    type SystemData = (
        ReadStorage<'a, Identifier>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Velocity>,
        ReadStorage<'a, Mass>,
    );

    fn run(&mut self, (id, positions, velocities, mass): Self::SystemData) {
        // Iterate over every entity
        (&id, &positions, &velocities, &mass)
            .join()
            .for_each(|(id, pos, vel, mass)| {
                // Print the entity's id, name, pos, vel and mass as an informational log
                info!(
                    "body{{id:{},name:{},pos:{:?},vel:{:?},mass:{:?}}}",
                    id.id, id.name, pos.0, vel.0, mass.0,
                );

                // Print the entity's name, pos, vel and mass as a debug log
                debug!(
                    "{} {{\n\tpos: {:?},\n\tvel: {:?},\n\tmass: {:?}\n}}",
                    id.name, pos.0, vel.0, mass.0,
                );
            });
    }
}
