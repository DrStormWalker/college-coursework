/*mod tab;

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

/// Enum to signify the change of a global state element
#[derive(Debug)]
pub enum GlobalState {
    ChangeGravitationalConstant(f64),
    ChangeScale(f64),
    ChangeCameraSpeed(f64),
    ChangeCameraPosition(VectorStateChange),
}

/// Enum to signify the change of a vectors component
#[derive(Debug)]
pub enum VectorStateChange {
    X(f64),
    Y(f64),
    Z(f64),
}

/// Enum to signify the change of an element specific to an orbital body
#[derive(Debug)]
pub enum BodyState {
    ChangePosition(VectorStateChange),
    ChangeVelocity(VectorStateChange),
    ChangeMass(f64),
}

/// Enum to signify the change of a state within the application used by the main
/// panel to handle user input.
#[derive(Debug)]
pub enum UiMessage {
    GlobalState(GlobalState),
    BodyState { id: String, state: BodyState },
}

/// Data structure to store the channels used by the panel to communicate with
/// the main thread
pub struct PanelChannels {
    pub sender_ui: Sender<UiMessage>,
    pub receiver_app: app::Receiver<UiMessage>,
}

/// Data structure that stores the instances of each input element in the UI panel
pub struct GlobalStateUi {
    gravitational_constant: FloatInput,
    scale: FloatInput,
    camera_speed: FloatInput,
    camera_position: VectorFloatInput,
}

/// structure that sets up and runs the UI panel
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
        //! Creates a new window for the UI and registers widgets for it

        // Create a new window
        let ui = fltk::app::App::default().with_scheme(Scheme::Gtk);
        let mut window = Window::default()
            .with_size(width, height)
            .with_label(title)
            .center_screen();

        // Unpack the channels given to be used by the panel to communicate with
        // the main thread
        let PanelChannels {
            sender_ui,
            receiver_app,
        } = channels;

        // Create the UI of the panel, by registering widgets
        let (global_state, tabs) = Self::register_ui(width, height, sender_ui.clone(), ids);

        // Make the window resizable
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
        //! Creates the panel UI, by registering widgets

        // Create a new section to contain the UI, spcaing each element appart by
        // 20 pixels
        let mut pack = Pack::new(20, 20, width - 30, height - 30, "");
        pack.set_spacing(20);

        // Create a widget for the Gravitation Constant
        let gravitational_constant = Flex::default()
            .with_size(width, 20)
            .with_label("Gravitational Constant")
            .row();
        let mut gravitational_constant_input = FloatInput::default();
        let sender = sender_ui.clone();

        // Communicate a change in value of the gravitational constant with the
        // main thread
        gravitational_constant_input.set_callback(move |input| {
            // Validate the new value to ensure it is a float
            let value: Result<f64, _> = input.value().parse();
            if let Ok(value) = value {
                let _ = sender.send(UiMessage::GlobalState(
                    GlobalState::ChangeGravitationalConstant(value),
                ));
            }
        });
        gravitational_constant.end();

        // Create a widget for the scale of the render
        let scale = Flex::default()
            .with_size(width, 20)
            .with_label("Position Scale Factor")
            .row();
        let mut scale_input = FloatInput::default();
        let sender = sender_ui.clone();

        // Communicate a change in the scale with the main thread
        scale_input.set_callback(move |input| {
            // Validate the new value to ensure it is a float
            let value: Result<f64, _> = input.value().parse();
            if let Ok(value) = value {
                let _ = sender.send(UiMessage::GlobalState(GlobalState::ChangeScale(value)));
            }
        });
        scale.end();

        // Create a widget fgor the position of the camera
        let mut camera_position = Flex::default()
            .with_size(width, 20)
            .with_label("Camera Position")
            .row();
        camera_position.set_pad(15);
        let mut camera_position_x = FloatInput::default().with_label("X");
        let mut camera_position_y = FloatInput::default().with_label("Y");
        let mut camera_position_z = FloatInput::default().with_label("Z");

        // Communicate a change in the position of the camera with the main thread
        let sender = sender_ui.clone();
        camera_position_x.set_callback(move |input| {
            // Validate the new value to ensure it is a float
            let value: Result<f64, _> = input.value().parse();
            if let Ok(value) = value {
                let _ = sender.send(UiMessage::GlobalState(GlobalState::ChangeCameraPosition(
                    VectorStateChange::X(value),
                )));
            }
        });

        let sender = sender_ui.clone();
        camera_position_y.set_callback(move |input| {
            // Validate the new value to ensure it is a float
            let value: Result<f64, _> = input.value().parse();
            if let Ok(value) = value {
                let _ = sender.send(UiMessage::GlobalState(GlobalState::ChangeCameraPosition(
                    VectorStateChange::Y(value),
                )));
            }
        });

        let sender = sender_ui.clone();
        camera_position_z.set_callback(move |input| {
            // Validate the new value to ensure it is a float
            let value: Result<f64, _> = input.value().parse();
            if let Ok(value) = value {
                let _ = sender.send(UiMessage::GlobalState(GlobalState::ChangeCameraPosition(
                    VectorStateChange::Z(value),
                )));
            }
        });
        camera_position.end();

        // Create a widget for the camera speed
        let camera_speed = Flex::default()
            .with_size(width, 20)
            .with_label("Camera Speed")
            .row();
        let mut camera_speed_input = FloatInput::default();

        // Communicate a change in the speed of the camera with the main thread
        let sender = sender_ui.clone();
        camera_speed_input.set_callback(move |input| {
            // Validate the new value to ensure it is a float
            let value: Result<f64, _> = input.value().parse();
            if let Ok(value) = value {
                let _ = sender.send(UiMessage::GlobalState(GlobalState::ChangeCameraSpeed(
                    value,
                )));
            }
        });
        camera_speed.end();

        // Create the tabs to hold the information about each planet
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

        // Return the registered inputs
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
        //! Runs the panel UI, handling messages sent by the main thread

        // Whow the UI window
        self.window.show();

        // Handle messages sent by the main thread
        while self.ui.wait() {
            // If the message actuall contains data
            if let Some(msg) = self.receiver_app.recv() {
                match msg {
                    UiMessage::BodyState { id, state } => match state {
                        BodyState::ChangePosition(component) => match component {
                            VectorStateChange::X(x) => self
                                .tabs
                                .iter_mut()
                                .filter(|tab| !tab.position.x.has_focus())
                                .filter(|tab| tab.id == id)
                                .for_each(|tab| {
                                    tab.position.x.set_value(&format!("{}", x / 1_000_000.0))
                                }),
                            VectorStateChange::Y(y) => self
                                .tabs
                                .iter_mut()
                                .filter(|tab| !tab.position.y.has_focus())
                                .filter(|tab| tab.id == id)
                                .for_each(|tab| {
                                    tab.position.y.set_value(&format!("{}", y / 1_000_000.0))
                                }),
                            VectorStateChange::Z(z) => self
                                .tabs
                                .iter_mut()
                                .filter(|tab| !tab.position.z.has_focus())
                                .filter(|tab| tab.id == id)
                                .for_each(|tab| {
                                    tab.position.z.set_value(&format!("{}", z / 1_000_000.0))
                                }),
                        },
                        BodyState::ChangeVelocity(component) => match component {
                            VectorStateChange::X(x) => self
                                .tabs
                                .iter_mut()
                                .filter(|tab| !tab.velocity.x.has_focus())
                                .filter(|tab| tab.id == id)
                                .for_each(|tab| {
                                    tab.velocity.x.set_value(&format!("{}", x / 1_000.0))
                                }),
                            VectorStateChange::Y(y) => self
                                .tabs
                                .iter_mut()
                                .filter(|tab| !tab.velocity.y.has_focus())
                                .filter(|tab| tab.id == id)
                                .for_each(|tab| {
                                    tab.velocity.y.set_value(&format!("{}", y / 1_000.0))
                                }),
                            VectorStateChange::Z(z) => self
                                .tabs
                                .iter_mut()
                                .filter(|tab| !tab.velocity.z.has_focus())
                                .filter(|tab| tab.id == id)
                                .for_each(|tab| {
                                    tab.velocity.z.set_value(&format!("{}", z / 1_000.0))
                                }),
                        },
                        BodyState::ChangeMass(mass) => self
                            .tabs
                            .iter_mut()
                            .filter(|tab| tab.id == id)
                            .for_each(|tab| tab.mass.set_value(&format!("{}", mass))),
                    },
                    UiMessage::GlobalState(state) => match state {
                        GlobalState::ChangeCameraSpeed(speed) => {
                            if !self.global_state.camera_speed.has_focus() {
                                self.global_state
                                    .camera_speed
                                    .set_value(&format!("{}", speed))
                            }
                        }
                        GlobalState::ChangeCameraPosition(component) => match component {
                            VectorStateChange::X(x) => {
                                if !self.global_state.camera_position.x.has_focus() {
                                    self.global_state
                                        .camera_position
                                        .x
                                        .set_value(&format!("{}", x))
                                }
                            }
                            VectorStateChange::Y(y) => {
                                if !self.global_state.camera_position.y.has_focus() {
                                    self.global_state
                                        .camera_position
                                        .y
                                        .set_value(&format!("{}", y))
                                }
                            }
                            VectorStateChange::Z(z) => {
                                if !self.global_state.camera_position.z.has_focus() {
                                    self.global_state
                                        .camera_position
                                        .z
                                        .set_value(&format!("{}", z))
                                }
                            }
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
}*/

mod global;
mod vector_ui;

pub use global::GlobalWindow;
pub use vector_ui::*;

pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui);
}

pub trait Window {
    fn name(&self) -> &'static str;

    fn show(&mut self, ctx: &egui::Context, open: &mut bool);
}
