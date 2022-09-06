use fltk::{
    group::{Flex, FlexType, Group, Pack, Tabs},
    input::FloatInput,
    prelude::*,
};

pub struct VectorFloatInput {
    x: FloatInput,
    y: FloatInput,
    z: FloatInput,
}

pub struct Tab {
    pub position: VectorFloatInput,
    pub velocity: VectorFloatInput,
    pub mass: FloatInput,
}
impl Tab {
    pub fn new(x: i32, y: i32, width: i32, height: i32, label: &str) -> Self {
        let group = Group::new(x, y, width, height, "").with_label(label);

        let mut pack = Pack::new(x + 5, y + 10, width - 5, height - 10, "");
        pack.set_spacing(20);

        let mut position = Flex::default()
            .with_size(width, 20)
            .with_label("Position")
            .row();
        position.set_pad(15);
        let pos_x = FloatInput::default().with_label("X");
        let pos_y = FloatInput::default().with_label("Y");
        let pos_z = FloatInput::default().with_label("Z");
        position.end();

        let mut velocity = Flex::default()
            .with_size(width, 20)
            .with_label("Velocity")
            .row();
        velocity.set_pad(15);
        let vel_x = FloatInput::default().with_label("X");
        let vel_y = FloatInput::default().with_label("Y");
        let vel_z = FloatInput::default().with_label("Z");
        velocity.end();

        let mass = Flex::default()
            .with_size(width, 20)
            .with_label("Mass")
            .row();
        let mass_mass = FloatInput::default();
        mass.end();

        pack.end();

        group.end();

        Self {
            position: VectorFloatInput {
                x: pos_x,
                y: pos_y,
                z: pos_z,
            },
            velocity: VectorFloatInput {
                x: vel_x,
                y: vel_y,
                z: vel_z,
            },
            mass: mass_mass,
        }
    }
}
