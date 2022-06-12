use specs::{Builder, Entity, World, WorldExt};

use super::{Mass, Position, Velocity};
use crate::util::Vec3;

pub struct OrbitalBody {
    initial_pos: [f64; 3],
    initial_vel: [f64; 3],
    mass: f64,
}
impl OrbitalBody {
    pub fn get_pos(&self) -> Position {
        Position::from(Vec3::from(self.initial_pos))
    }

    pub fn get_vel(&self) -> Velocity {
        Velocity::from(Vec3::from(self.initial_vel))
    }

    pub fn get_mass(&self) -> Mass {
        Mass::from(self.mass)
    }

    pub fn register_entity(&self, world: &mut World) -> Entity {
        world
            .create_entity()
            .with(self.get_pos())
            .with(self.get_vel())
            .with(self.get_mass())
            .build()
    }
}

pub const SUN: OrbitalBody = OrbitalBody {
    initial_pos: [0.0, 0.0, 0.0],
    initial_vel: [0.0, 0.0, 0.0],
    mass: 1.989e30,
};

pub const PLANET_MERCURY: OrbitalBody = OrbitalBody {
    initial_pos: [57.909e9, 0.0, 0.0],
    initial_vel: [0.0, 47.36e3, 0.0],
    mass: 0.33011e24,
};

pub const PLANET_VENUS: OrbitalBody = OrbitalBody {
    initial_pos: [108.209e9, 0.0, 0.0],
    initial_vel: [0.0, 35.02e3, 0.0],
    mass: 4.8675e24,
};

pub const PLANET_EARTH: OrbitalBody = OrbitalBody {
    initial_pos: [149.596e9, 0.0, 0.0],
    initial_vel: [0.0, 29.78e3, 0.0],
    mass: 5.9724e24,
};

pub const PLANET_MARS: OrbitalBody = OrbitalBody {
    initial_pos: [227.923e9, 0.0, 0.0],
    initial_vel: [0.0, 24.07e3, 0.0],
    mass: 0.64171e24,
};

pub const PLANET_JUPITER: OrbitalBody = OrbitalBody {
    initial_pos: [778.570e9, 0.0, 0.0],
    initial_vel: [0.0, 13e3, 0.0],
    mass: 1898.19e24,
};

pub const PLANET_SATURN: OrbitalBody = OrbitalBody {
    initial_pos: [1433.529e9, 0.0, 0.0],
    initial_vel: [0.0, 9.68e3, 0.0],
    mass: 568.34e24,
};

pub const PLANET_URANUS: OrbitalBody = OrbitalBody {
    initial_pos: [2872.463e9, 0.0, 0.0],
    initial_vel: [0.0, 6.80e3, 0.0],
    mass: 86.813e24,
};

pub const PLANET_NEPTUNE: OrbitalBody = OrbitalBody {
    initial_pos: [4495.060e9, 0.0, 0.0],
    initial_vel: [0.0, 5.43e3, 0.0],
    mass: 102.413e24,
};
