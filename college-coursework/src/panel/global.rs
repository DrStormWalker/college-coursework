use cgmath::Vector3;
use chrono::{DateTime, Local, Utc};
use egui::RichText;

use crate::{
    simulation::{Identifier, SUN},
    util::{convert_datetime_to_julian_date, convert_julian_date_to_datetime},
};

use super::{
    dynamic_decimals_formatter, dynamic_exponent_formatter, help::HelpWindow, planet::PlanetWindow,
    DateTimeValue, Vector3Value,
};

pub const MINUS_EXPONENT: &'static str = "\u{2C9}";
pub const ONE_EXPONENT: &'static str = "\u{F80B}";
pub const MINUS_ONE_EXPONENT: &'static str = const_format::concatcp!(MINUS_EXPONENT, ONE_EXPONENT);
pub const TWO_EXPONENT: &'static str = "\u{F80C}";
pub const MINUS_TWO_EXPONENT: &'static str = const_format::concatcp!(MINUS_EXPONENT, TWO_EXPONENT);

#[derive(PartialEq)]
pub enum CameraControllerType {
    Free,
    Orbit,
}

pub struct GlobalWindow<'a> {
    pub camera_section: CameraSection<'a>,
    pub constant_section: ConstantSection<'a>,
    pub time_section: TimeSection<'a>,
    pub help_window_shown: &'a mut bool,
    pub planet_windows_shown: Vec<(Identifier, &'a mut bool)>,
    pub save_window_shown: &'a mut bool,
    pub load_window_shown: &'a mut bool,
}
impl<'a> super::View for GlobalWindow<'a> {
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.camera_section.ui(ui);
        self.constant_section.ui(ui);
        self.time_section.ui(ui);

        egui::CollapsingHeader::new("Bodies")
            .default_open(false)
            .show(ui, |ui| {
                for (id, shown) in self.planet_windows_shown.iter_mut() {
                    if ui.button(id.get_name()).clicked() {
                        **shown = !**shown;
                    }
                }
            });

        ui.separator();

        ui.vertical_centered(|ui| {
            if ui.link("Save Simulation").clicked() {
                *self.save_window_shown = !*self.save_window_shown;
            }

            if ui.link("Load Simulation").clicked() {
                *self.load_window_shown = !*self.load_window_shown;
            }
        });

        ui.vertical_centered(|ui| {
            if ui.link("Help").clicked() {
                *self.help_window_shown = !*self.help_window_shown;
            }
        });
    }
}
impl<'a> super::Window for GlobalWindow<'a> {
    fn name(&self) -> &'static str {
        "Global State"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        use super::View as _;
        egui::Window::new(self.name())
            .collapsible(true)
            .resizable(true)
            //.open(open)
            .show(ctx, |ui| self.ui(ui));
    }
}

pub struct CameraSection<'a> {
    pub position: &'a mut Vector3<f32>,
    pub speed: &'a mut f32,
    pub controller_type: &'a mut CameraControllerType,
}
impl<'a> super::View for CameraSection<'a> {
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Camera")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Position:");
                    ui.add(
                        Vector3Value::new(self.position)
                            .custom_formatter(dynamic_exponent_formatter())
                            .suffix(" m")
                            .speed(0.1),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Speed:");
                    ui.add(
                        egui::Slider::new(self.speed, 1.0..=1_000_000.0)
                            .logarithmic(true)
                            .suffix(const_format::concatcp!(" ms", MINUS_ONE_EXPONENT))
                            .step_by(0.1)
                            .custom_formatter(dynamic_exponent_formatter()),
                    )
                });

                ui.horizontal(|ui| {
                    ui.label("Controller:");
                    ui.selectable_value(self.controller_type, CameraControllerType::Free, "Free");
                    ui.selectable_value(self.controller_type, CameraControllerType::Orbit, "Orbit");
                });
            });
    }
}

pub struct ConstantSection<'a> {
    pub gravitational_constant: &'a mut f64,
}
impl<'a> super::View for ConstantSection<'a> {
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Constants")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Gravitational Constant:");
                    ui.add(
                        egui::DragValue::new(self.gravitational_constant)
                            .clamp_range(0.0..=f64::INFINITY)
                            .speed(0.01e-11)
                            .custom_formatter(dynamic_exponent_formatter())
                            .suffix(const_format::concatcp!(
                                " Nm",
                                TWO_EXPONENT,
                                "kg",
                                MINUS_TWO_EXPONENT
                            )),
                    );
                })
            });
    }
}

pub struct TimeSection<'a> {
    pub time_scale: &'a mut f64,
    pub current_date_time: &'a mut DateTime<Local>,
}
impl<'a> super::View for TimeSection<'a> {
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Time")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Time Scale:");
                    ui.add(
                        egui::Slider::new(self.time_scale, 0.0..=3_155_760_000.0)
                            .logarithmic(true)
                            .custom_formatter(dynamic_exponent_formatter()),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Date:");

                    let mut date = self.current_date_time.date().with_timezone(&Utc);
                    ui.add(egui_extras::DatePickerButton::new(&mut date));
                    *self.current_date_time = date
                        .with_timezone(&Local)
                        .and_time(self.current_date_time.time())
                        .unwrap();

                    ui.label("Time:");
                    ui.add(DateTimeValue::new("time_value", self.current_date_time));
                });

                ui.horizontal(|ui| {
                    ui.label("Julian Date:");
                    let mut julian_date = convert_datetime_to_julian_date(
                        &self.current_date_time.with_timezone(&Utc),
                    );
                    let response = ui.add(
                        egui::DragValue::new(&mut julian_date)
                            .speed(0.1)
                            .custom_formatter(dynamic_decimals_formatter()),
                    );
                    if response.changed() {
                        *self.current_date_time =
                            convert_julian_date_to_datetime(julian_date).with_timezone(&Local);
                    }
                });
            });
    }
}
