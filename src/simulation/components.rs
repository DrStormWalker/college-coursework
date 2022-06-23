use log::{debug, info};
use specs::{Component, Join, ReadStorage, System, VecStorage};

use crate::util::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Position(pub Vec3);
impl From<Vec3> for Position {
    fn from(v: Vec3) -> Self {
        Self(v)
    }
}
impl Component for Position {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone, Copy)]
pub struct Velocity(pub Vec3);
impl From<Vec3> for Velocity {
    fn from(v: Vec3) -> Self {
        Self(v)
    }
}
impl Component for Velocity {
    type Storage = VecStorage<Self>;
}

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

#[derive(Default, Copy, Clone)]
pub struct DeltaTime(pub f64);

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
        (&id, &positions, &velocities, &mass)
            .join()
            .for_each(|(id, pos, vel, mass)| {
                info!(
                    "body{{id:{},name:{},pos:{:?},vel:{:?},mass:{:?}}}",
                    id.id, id.name, pos.0, vel.0, mass.0,
                );

                debug!(
                    "{} {{\n\tpos: {:?},\n\tvel: {:?},\n\tmass: {:?}\n}}",
                    id.name, pos.0, vel.0, mass.0,
                );
            });
    }
}

/*
#[derive(Debug, Clone, Copy)]
pub struct SimIdentifier(usize);
impl SimIdentifier {
    pub fn new(i: usize) -> Self {
        Self(i)
    }

    pub fn id(&self) -> usize {
        self.0
    }
}
impl Component for SimIdentifier {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Default)]
pub struct Positions {
    pub current: Vec<Vec3>,
    pub next: Vec<Vec3>,
}
impl Positions {
    pub fn new(current: Vec<Vec3>, next: Vec<Vec3>) -> Self {
        Self { current, next }
    }
}

#[derive(Debug, Default)]
pub struct Velocities {
    pub current: Vec<Vec3>,
    pub next: Vec<Vec3>,
}
impl Velocities {
    pub fn new(current: Vec<Vec3>, next: Vec<Vec3>) -> Self {
        Self { current, next }
    }
}

#[derive(Debug, Default)]
pub struct Masses(pub Vec<f64>);
impl Masses {
    pub fn new(masses: Vec<f64>) -> Self {
        Self(masses)
    }
}
*/
