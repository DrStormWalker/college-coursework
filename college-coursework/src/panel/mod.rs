mod tab;

use crossbeam::channel::{unbounded, Receiver, Sender};
use fltk::{
    app::Scheme,
    group::{Flex, Pack, Tabs},
    input::FloatInput,
    prelude::*,
    window::Window,
};
use nalgebra::{Point3, Vector3};

use crate::simulation::Identifier;

pub enum UiGlobalState {
    ChangeGravitationalConstant(f32),
    ChangeScale(f32),
    ChangeCameraSpeed(f32),
    ChangeCameraPosition(f32),
}

pub enum UiBodyState {
    ChangePosition(Point3<f32>),
    ChangeVelocity(Vector3<f32>),
    ChangeMass(f32),
}

pub enum UiMessage {
    GlobalState(UiGlobalState),
    BodyState { id: Identifier, state: UiBodyState },
}

pub enum AppGlobalState {
    ChangeCameraSpeed(f32),
    ChangeCameraPosition(Point3<f32>),
}

pub enum AppBodyState {
    ChangePosition(Point3<f32>),
    ChangeVelocity(Vector3<f32>),
}

pub enum AppMessage {
    GlobalState(AppGlobalState),
    BodyState { id: Identifier, state: AppBodyState },
}

pub struct PanelChannels {
    sender_app: Sender<AppMessage>,
    receiver_ui: Receiver<UiMessage>,
}

pub struct Ui {
    ui: fltk::app::App,
    window: Window,
    sender_ui: Sender<UiMessage>,
    receiver_app: Receiver<AppMessage>,
}
impl Ui {
    pub fn new(width: i32, height: i32, title: &str) -> (Self, PanelChannels) {
        let ui = fltk::app::App::default().with_scheme(Scheme::Gtk);
        let mut window = Window::default()
            .with_size(width, height)
            .with_label(title)
            .center_screen();

        Self::register_ui(width, height);

        window.make_resizable(true);
        window.end();

        let (sender_ui, receiver_ui) = unbounded();
        let (sender_app, receiver_app) = unbounded();

        (
            Self {
                ui,
                window,
                sender_ui,
                receiver_app,
            },
            PanelChannels {
                sender_app,
                receiver_ui,
            },
        )
    }

    fn register_ui(width: i32, height: i32) {
        let mut pack = Pack::new(20, 20, width - 30, height - 30, "");
        pack.set_spacing(20);

        let gravitational_constant = Flex::default()
            .with_size(width, 20)
            .with_label("Gravitational Constant")
            .row();
        let _gravitational_constant = FloatInput::default();
        gravitational_constant.end();

        let scale = Flex::default()
            .with_size(width, 20)
            .with_label("Position Scale Factor")
            .row();
        let _scale = FloatInput::default();
        scale.end();

        let mut camera_position = Flex::default()
            .with_size(width, 20)
            .with_label("Camera Position")
            .row();
        camera_position.set_pad(15);
        let _camera_position_x = FloatInput::default().with_label("X");
        let _camera_position_y = FloatInput::default().with_label("Y");
        let _camera_position_z = FloatInput::default().with_label("Z");
        camera_position.end();

        let camera_speed = Flex::default()
            .with_size(width, 20)
            .with_label("Camera Speed")
            .row();
        let _camera_speed = FloatInput::default();
        camera_speed.end();

        let tab = Tabs::new(10, height / 2 + 10, width - 20, height / 2 - 20, "");
        for i in 1..3 {
            tab::Tab::new(
                10,
                height / 2 + 25,
                width - 20,
                height - 45,
                &format!("Tab {:?}", i),
            );
        }
        tab.end();

        pack.end();
    }

    pub fn run(mut self) {
        self.window.show();

        while self.ui.wait() {}
    }
}
