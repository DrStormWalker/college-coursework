use log::info;
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
        ReadStorage<'a, Position>,
        ReadStorage<'a, Velocity>,
        ReadStorage<'a, Mass>,
    );

    fn run(&mut self, (positions, velocities, mass): Self::SystemData) {
        (&positions, &velocities, &mass)
            .join()
            .for_each(|(pos, vel, mass)| {
                info!(
                    "body {{ pos: {:?}, vel: {:?}, mass: {:?} }}",
                    pos, vel, mass
                )
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
