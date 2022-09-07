use cgmath::Vector3;
use specs::{Builder, Component, Entity, VecStorage, World, WorldExt};

use super::{Identifier, Mass, Position, Velocity};
use crate::util::Vec3;

use bitflags::bitflags;

#[derive(Debug, Copy, Clone)]
pub enum BodyType {
    Star,
    Planet,
}

bitflags! {
    pub struct InteractionFlags: u32 {
        const STAR = 1 << BodyType::Star as u32;
        const PLANET = 1 << BodyType::Planet as u32;
    }
}
impl From<BodyType> for InteractionFlags {
    fn from(body_type: BodyType) -> Self {
        InteractionFlags::from_bits(1 << body_type as u32).unwrap()
    }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct InteractionHandler {
    pub flags: InteractionFlags,
    pub body_type: BodyType,
}
impl InteractionHandler {
    pub fn new(flags: InteractionFlags, body_type: BodyType) -> Self {
        Self { flags, body_type }
    }
}

// A structure to contain the information about an orbital body
pub struct OrbitalBody {
    id: &'static str,
    name: &'static str,
    initial_pos: [f64; 3],
    initial_vel: [f64; 3],
    colour: [f32; 4],
    mass: f64,
    body_type: BodyType,
}
impl OrbitalBody {
    pub fn get_pos(&self) -> Position {
        Position::from(Vector3::<f64>::from(self.initial_pos))
    }

    pub fn get_vel(&self) -> Velocity {
        Velocity::from(Vector3::<f64>::from(self.initial_vel))
    }

    pub fn get_mass(&self) -> Mass {
        Mass::from(self.mass)
    }

    pub fn get_identifier(&self) -> Identifier {
        Identifier::new(self.id.to_string(), self.name.to_string())
    }

    pub fn get_colour(&self) -> [f32; 4] {
        self.colour
    }

    pub fn register_entity(&self, world: &mut World) -> Entity {
        // Register the entity into the ECS world
        world
            .create_entity()
            .with(self.get_identifier())
            .with(self.get_pos())
            .with(self.get_vel())
            .with(self.get_mass())
            .build()
    }
}

// The hardcoded initial positions for each planet and the sun
// In later iterations these values will be calculated using
// data gathered about the orbits of these planets
pub const SUN: OrbitalBody = OrbitalBody {
    id: "sun",
    name: "Sun",
    initial_pos: [0.0, 0.0, 0.0],
    initial_vel: [0.0, 0.0, 0.0],
    colour: [252.0 / 255.0, 229.0 / 255.0, 112.0 / 255.0, 1.0],
    mass: 1.989e30,
    body_type: BodyType::Star,
};

pub const PLANET_MERCURY: OrbitalBody = OrbitalBody {
    id: "mercury",
    name: "Mercury",
    initial_pos: [57.909e9, 0.0, 0.0],
    initial_vel: [0.0, 47.36e3, 0.0],
    colour: [0.7, 0.7, 0.7, 1.0],
    mass: 0.33011e24,
    body_type: BodyType::Planet,
};

pub const PLANET_VENUS: OrbitalBody = OrbitalBody {
    id: "venus",
    name: "Venus",
    initial_pos: [108.209e9, 0.0, 0.0],
    initial_vel: [0.0, 35.02e3, 0.0],
    colour: [0.9, 0.9, 0.9, 1.0],
    mass: 4.8675e24,
    body_type: BodyType::Planet,
};

pub const PLANET_EARTH: OrbitalBody = OrbitalBody {
    id: "earth",
    name: "Earth",
    initial_pos: [149.596e9, 0.0, 0.0],
    initial_vel: [0.0, 29.78e3, 0.0],
    colour: [0.0, 1.0, 0.0, 1.0],
    mass: 5.9724e24,
    body_type: BodyType::Planet,
};

pub const PLANET_MARS: OrbitalBody = OrbitalBody {
    id: "mars",
    name: "Mars",
    initial_pos: [227.923e9, 0.0, 0.0],
    initial_vel: [0.0, 24.07e3, 0.0],
    colour: [1.0, 0.0, 0.0, 1.0],
    mass: 0.64171e24,
    body_type: BodyType::Planet,
};

pub const PLANET_JUPITER: OrbitalBody = OrbitalBody {
    id: "jupiter",
    name: "Jupiter",
    initial_pos: [778.570e9, 0.0, 0.0],
    initial_vel: [0.0, 13e3, 0.0],
    colour: [0.605, 0.428, 0.299, 1.0],
    mass: 1898.19e24,
    body_type: BodyType::Planet,
};

pub const PLANET_SATURN: OrbitalBody = OrbitalBody {
    id: "saturn",
    name: "Saturn",
    initial_pos: [1433.529e9, 0.0, 0.0],
    initial_vel: [0.0, 9.68e3, 0.0],
    colour: [0.605, 0.428, 0.399, 1.0],
    mass: 568.34e24,
    body_type: BodyType::Planet,
};

pub const PLANET_URANUS: OrbitalBody = OrbitalBody {
    id: "uranus",
    name: "Uranus",
    initial_pos: [2872.463e9, 0.0, 0.0],
    initial_vel: [0.0, 6.80e3, 0.0],
    colour: [0.0, 0.5, 1.0, 1.0],
    mass: 86.813e24,
    body_type: BodyType::Planet,
};

pub const PLANET_NEPTUNE: OrbitalBody = OrbitalBody {
    id: "neptune",
    name: "Neptune",
    initial_pos: [4495.060e9, 0.0, 0.0],
    initial_vel: [0.0, 5.43e3, 0.0],
    colour: [0.0, 0.0, 1.0, 1.0],
    mass: 102.413e24,
    body_type: BodyType::Planet,
};

pub fn planets() -> Vec<OrbitalBody> {
    vec![
        PLANET_MERCURY,
        PLANET_VENUS,
        PLANET_EARTH,
        PLANET_MARS,
        PLANET_JUPITER,
        PLANET_SATURN,
        PLANET_URANUS,
        PLANET_NEPTUNE,
    ]
}
