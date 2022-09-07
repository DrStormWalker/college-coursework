mod tab;

use crossbeam::channel::{unbounded, Receiver, Sender};
use fltk::{
    app::{self, Scheme},
    group::{Flex, Pack, Tabs},
    input::FloatInput,
    prelude::*,
    window::Window,
};
use nalgebra::{Point3, Vector3};

use crate::simulation::Identifier;

use self::tab::{Tab, VectorFloatInput};

#[derive(Debug)]
pub enum GlobalState {
    ChangeGravitationalConstant(f64),
    ChangeScale(f64),
    ChangeCameraSpeed(f64),
    ChangeCameraPosition(VectorStateChange),
}

#[derive(Debug)]
pub enum VectorStateChange {
    X(f64),
    Y(f64),
    Z(f64),
}

#[derive(Debug)]
pub enum BodyState {
    ChangePosition(VectorStateChange),
    ChangeVelocity(VectorStateChange),
    ChangeMass(f64),
}

#[derive(Debug)]
pub enum UiMessage {
    GlobalState(GlobalState),
    BodyState { id: String, state: BodyState },
}

pub struct PanelChannels {
    pub sender_ui: Sender<UiMessage>,
    pub receiver_app: app::Receiver<UiMessage>,
}

pub struct GlobalStateUi {
    gravitational_constant: FloatInput,
    scale: FloatInput,
    camera_speed: FloatInput,
    camera_position: VectorFloatInput,
}

pub struct Ui {
    ui: fltk::app::App,
    window: Window,
    sender_ui: Sender<UiMessage>,
    receiver_app: app::Receiver<UiMessage>,
    global_state: GlobalStateUi,
    tabs: Vec<Tab>,
}
impl Ui {
    pub fn new(
        width: i32,
        height: i32,
        title: &str,
        channels: PanelChannels,
        ids: Vec<Identifier>,
    ) -> Self {
        let ui = fltk::app::App::default().with_scheme(Scheme::Gtk);
        let mut window = Window::default()
            .with_size(width, height)
            .with_label(title)
            .center_screen();

        let PanelChannels {
            sender_ui,
            receiver_app,
        } = channels;

        let (global_state, tabs) = Self::register_ui(width, height, sender_ui.clone(), ids);

        window.make_resizable(true);
        window.end();

        Self {
            ui,
            window,
            sender_ui,
            receiver_app,
            global_state,
            tabs,
        }
    }

    fn register_ui(
        width: i32,
        height: i32,
        sender_ui: Sender<UiMessage>,
        ids: Vec<Identifier>,
    ) -> (GlobalStateUi, Vec<Tab>) {
        let mut pack = Pack::new(20, 20, width - 30, height - 30, "");
        pack.set_spacing(20);

        let gravitational_constant = Flex::default()
            .with_size(width, 20)
            .with_label("Gravitational Constant")
            .row();
        let mut gravitational_constant_input = FloatInput::default();
        let sender = sender_ui.clone();
        gravitational_constant_input.set_callback(move |input| {
            let value: Result<f64, _> = input.value().parse();
            if let Ok(value) = value {
                let _ = sender.send(UiMessage::GlobalState(
                    GlobalState::ChangeGravitationalConstant(value),
                ));
            }
        });
        gravitational_constant.end();

        let scale = Flex::default()
            .with_size(width, 20)
            .with_label("Position Scale Factor")
            .row();
        let mut scale_input = FloatInput::default();
        let sender = sender_ui.clone();
        scale_input.set_callback(move |input| {
            let value: Result<f64, _> = input.value().parse();
            if let Ok(value) = value {
                let _ = sender.send(UiMessage::GlobalState(GlobalState::ChangeScale(value)));
            }
        });
        scale.end();

        let mut camera_position = Flex::default()
            .with_size(width, 20)
            .with_label("Camera Position")
            .row();
        camera_position.set_pad(15);
        let mut camera_position_x = FloatInput::default().with_label("X");
        let mut camera_position_y = FloatInput::default().with_label("Y");
        let mut camera_position_z = FloatInput::default().with_label("Z");

        let sender = sender_ui.clone();
        camera_position_x.set_callback(move |input| {
            let value: Result<f64, _> = input.value().parse();
            if let Ok(value) = value {
                let _ = sender.send(UiMessage::GlobalState(GlobalState::ChangeCameraPosition(
                    VectorStateChange::X(value),
                )));
            }
        });

        let sender = sender_ui.clone();
        camera_position_y.set_callback(move |input| {
            let value: Result<f64, _> = input.value().parse();
            if let Ok(value) = value {
                let _ = sender.send(UiMessage::GlobalState(GlobalState::ChangeCameraPosition(
                    VectorStateChange::Y(value),
                )));
            }
        });

        let sender = sender_ui.clone();
        camera_position_z.set_callback(move |input| {
            let value: Result<f64, _> = input.value().parse();
            if let Ok(value) = value {
                let _ = sender.send(UiMessage::GlobalState(GlobalState::ChangeCameraPosition(
                    VectorStateChange::Z(value),
                )));
            }
        });
        camera_position.end();

        let camera_speed = Flex::default()
            .with_size(width, 20)
            .with_label("Camera Speed")
            .row();
        let mut camera_speed_input = FloatInput::default();
        let sender = sender_ui.clone();
        camera_speed_input.set_callback(move |input| {
            let value: Result<f64, _> = input.value().parse();
            if let Ok(value) = value {
                let _ = sender.send(UiMessage::GlobalState(GlobalState::ChangeCameraSpeed(
                    value,
                )));
            }
        });
        camera_speed.end();

        let tab = Tabs::new(10, height / 2 + 10, width - 20, height / 2 - 20, "");
        let tabs = ids
            .into_iter()
            .map(|id| {
                tab::Tab::new(
                    10,
                    height / 2 + 25,
                    width - 20,
                    height - 45,
                    &format!("{}", id.get_name()),
                    sender_ui.clone(),
                    id.get_id().to_string(),
                )
            })
            .collect::<Vec<_>>();
        tab.end();

        pack.end();

        (
            GlobalStateUi {
                gravitational_constant: gravitational_constant_input,
                scale: scale_input,
                camera_position: VectorFloatInput {
                    x: camera_position_x,
                    y: camera_position_y,
                    z: camera_position_z,
                },
                camera_speed: camera_speed_input,
            },
            tabs,
        )
    }

    pub fn run(mut self) {
        self.window.show();

        while self.ui.wait() {
            if let Some(msg) = self.receiver_app.recv() {
                match msg {
                    UiMessage::BodyState { id, state } => {
                        match state {
                            BodyState::ChangePosition(component) => match component {
                                VectorStateChange::X(x) => self
                                    .tabs
                                    .iter_mut()
                                    .filter(|tab| tab.id == id)
                                    .for_each(|tab| {
                                        tab.position.x.set_value(&format!("{}", x / 1_000_000.0))
                                    }),
                                VectorStateChange::Y(y) => self
                                    .tabs
                                    .iter_mut()
                                    .filter(|tab| tab.id == id)
                                    .for_each(|tab| {
                                        tab.position.y.set_value(&format!("{}", y / 1_000_000.0))
                                    }),
                                VectorStateChange::Z(z) => self
                                    .tabs
                                    .iter_mut()
                                    .filter(|tab| tab.id == id)
                                    .for_each(|tab| {
                                        tab.position.z.set_value(&format!("{}", z / 1_000_000.0))
                                    }),
                            },
                            BodyState::ChangeVelocity(component) => match component {
                                VectorStateChange::X(x) => {
                                    self.tabs.iter_mut().filter(|tab| tab.id == id).for_each(
                                        |tab| tab.velocity.x.set_value(&format!("{}", x / 1_000.0)),
                                    )
                                }
                                VectorStateChange::Y(y) => {
                                    self.tabs.iter_mut().filter(|tab| tab.id == id).for_each(
                                        |tab| tab.velocity.y.set_value(&format!("{}", y / 1_000.0)),
                                    )
                                }
                                VectorStateChange::Z(z) => {
                                    self.tabs.iter_mut().filter(|tab| tab.id == id).for_each(
                                        |tab| tab.velocity.z.set_value(&format!("{}", z / 1_000.0)),
                                    )
                                }
                            },
                            BodyState::ChangeMass(mass) => self
                                .tabs
                                .iter_mut()
                                .filter(|tab| tab.id == id)
                                .for_each(|tab| tab.mass.set_value(&format!("{}", mass))),
                        }
                    }
                    UiMessage::GlobalState(state) => match state {
                        GlobalState::ChangeCameraSpeed(speed) => self
                            .global_state
                            .camera_speed
                            .set_value(&format!("{}", speed)),
                        GlobalState::ChangeCameraPosition(component) => match component {
                            VectorStateChange::X(x) => self
                                .global_state
                                .camera_position
                                .x
                                .set_value(&format!("{}", x)),
                            VectorStateChange::Y(y) => self
                                .global_state
                                .camera_position
                                .y
                                .set_value(&format!("{}", y)),
                            VectorStateChange::Z(z) => self
                                .global_state
                                .camera_position
                                .z
                                .set_value(&format!("{}", z)),
                        },
                        GlobalState::ChangeGravitationalConstant(constant) => self
                            .global_state
                            .gravitational_constant
                            .set_value(&format!("{}", constant)),
                        GlobalState::ChangeScale(scale) => {
                            self.global_state.scale.set_value(&format!("{}", scale))
                        }
                    },
                }
            }
        }
    }
}
