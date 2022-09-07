use crossbeam::channel::Sender;
use fltk::{
    group::{Flex, FlexType, Group, Pack, Tabs},
    input::FloatInput,
    prelude::*,
};

use crate::simulation::Identifier;

use super::{BodyState, UiMessage, VectorStateChange};

pub struct VectorFloatInput {
    pub x: FloatInput,
    pub y: FloatInput,
    pub z: FloatInput,
}

pub struct Tab {
    pub id: String,
    pub position: VectorFloatInput,
    pub velocity: VectorFloatInput,
    pub mass: FloatInput,
}
impl Tab {
    pub fn new(
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        label: &str,
        sender_ui: Sender<UiMessage>,
        id_string: String,
    ) -> Self {
        let group = Group::new(x, y, width, height, "").with_label(label);

        let mut pack = Pack::new(x + 5, y + 25, width - 5, height - 10, "");
        pack.set_spacing(20);

        let mut position = Flex::default()
            .with_size(width, 20)
            .with_label("Position")
            .row();
        position.set_pad(15);
        let mut pos_x = FloatInput::default().with_label("X");
        let mut pos_y = FloatInput::default().with_label("Y");
        let mut pos_z = FloatInput::default().with_label("Z");

        let sender = sender_ui.clone();
        let id = id_string.clone();
        pos_x.set_callback(move |input| {
            let value: Result<f64, _> = input.value().parse();
            if let Ok(value) = value {
                let _ = sender.send(UiMessage::BodyState {
                    id: id.clone(),
                    state: BodyState::ChangePosition(VectorStateChange::X(value)),
                });
            }
        });

        let sender = sender_ui.clone();
        let id = id_string.clone();
        pos_y.set_callback(move |input| {
            let value: Result<f64, _> = input.value().parse();
            if let Ok(value) = value {
                let _ = sender.send(UiMessage::BodyState {
                    id: id.clone(),
                    state: BodyState::ChangePosition(VectorStateChange::Y(value)),
                });
            }
        });

        let sender = sender_ui.clone();
        let id = id_string.clone();
        pos_z.set_callback(move |input| {
            let value: Result<f64, _> = input.value().parse();
            if let Ok(value) = value {
                let _ = sender.send(UiMessage::BodyState {
                    id: id.clone(),
                    state: BodyState::ChangePosition(VectorStateChange::Z(value)),
                });
            }
        });

        let sender = sender_ui.clone();
        let id = id_string.clone();
        position.end();

        let mut velocity = Flex::default()
            .with_size(width, 20)
            .with_label("Velocity")
            .row();
        velocity.set_pad(15);
        let mut vel_x = FloatInput::default().with_label("X");
        let mut vel_y = FloatInput::default().with_label("Y");
        let mut vel_z = FloatInput::default().with_label("Z");

        let sender = sender_ui.clone();
        let id = id_string.clone();
        vel_x.set_callback(move |input| {
            let value: Result<f64, _> = input.value().parse();
            if let Ok(value) = value {
                let _ = sender.send(UiMessage::BodyState {
                    id: id.clone(),
                    state: BodyState::ChangeVelocity(VectorStateChange::X(value)),
                });
            }
        });

        let sender = sender_ui.clone();
        let id = id_string.clone();
        vel_y.set_callback(move |input| {
            let value: Result<f64, _> = input.value().parse();
            if let Ok(value) = value {
                let _ = sender.send(UiMessage::BodyState {
                    id: id.clone(),
                    state: BodyState::ChangeVelocity(VectorStateChange::Y(value)),
                });
            }
        });

        let sender = sender_ui.clone();
        let id = id_string.clone();
        vel_z.set_callback(move |input| {
            let value: Result<f64, _> = input.value().parse();
            if let Ok(value) = value {
                let _ = sender.send(UiMessage::BodyState {
                    id: id.clone(),
                    state: BodyState::ChangeVelocity(VectorStateChange::Z(value)),
                });
            }
        });

        velocity.end();

        let mass = Flex::default()
            .with_size(width, 20)
            .with_label("Mass")
            .row();
        let mut mass_input = FloatInput::default();

        let sender = sender_ui.clone();
        let id = id_string.clone();
        mass_input.set_callback(move |input| {
            let value: Result<f64, _> = input.value().parse();
            if let Ok(value) = value {
                let _ = sender.send(UiMessage::BodyState {
                    id: id.clone(),
                    state: BodyState::ChangeMass(value),
                });
            }
        });

        mass.end();

        pack.end();

        group.end();

        Self {
            id: id_string,
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
            mass: mass_input,
        }
    }
}
