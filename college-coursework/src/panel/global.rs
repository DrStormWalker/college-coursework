use chrono::{DateTime, Local, Utc};

use super::{dynamic_exponent_formatter, DateTimeValue, Vector3Value};

const MINUS_EXPONENT: &'static str = "\u{2C9}";
const ONE_EXPONENT: &'static str = "\u{F80B}";
const MINUS_ONE_EXPONENT: &'static str = const_format::concatcp!(MINUS_EXPONENT, ONE_EXPONENT);
const TWO_EXPONENT: &'static str = "\u{F80C}";
const MINUS_TWO_EXPONENT: &'static str = const_format::concatcp!(MINUS_EXPONENT, TWO_EXPONENT);

#[derive(PartialEq)]
pub enum CameraControllerType {
    Free,
    Orbit,
}

pub struct GlobalWindow {
    camera_section: CameraSection,
    constant_section: ConstantSection,
    time_section: TimeSection,
}
impl Default for GlobalWindow {
    fn default() -> Self {
        Self {
            camera_section: CameraSection::default(),
            constant_section: ConstantSection::default(),
            time_section: TimeSection::default(),
        }
    }
}
impl super::View for GlobalWindow {
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.camera_section.ui(ui);
        self.constant_section.ui(ui);
        self.time_section.ui(ui);
    }
}
impl super::Window for GlobalWindow {
    fn name(&self) -> &'static str {
        "Global State"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        use super::View as _;
        egui::Window::new(self.name())
            .collapsible(true)
            .resizable(true)
            .open(open)
            .show(ctx, |ui| self.ui(ui));
    }
}

pub struct CameraSection {
    position: [f64; 3],
    speed: f64,
    controller_type: CameraControllerType,
}
impl Default for CameraSection {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            speed: 0.0,
            controller_type: CameraControllerType::Orbit,
        }
    }
}
impl super::View for CameraSection {
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Camera")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Position:");
                    ui.add(
                        Vector3Value::new(&mut self.position)
                            .custom_formatter(dynamic_exponent_formatter())
                            .suffix(" m")
                            .speed(0.1),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Speed:");
                    ui.add(
                        egui::Slider::new(&mut self.speed, 1.0..=1_000_000.0)
                            .logarithmic(true)
                            .suffix(const_format::concatcp!(" ms", MINUS_ONE_EXPONENT))
                            .step_by(0.1)
                            .custom_formatter(dynamic_exponent_formatter()),
                    )
                });

                ui.horizontal(|ui| {
                    ui.label("Controller:");
                    ui.selectable_value(
                        &mut self.controller_type,
                        CameraControllerType::Free,
                        "Free",
                    );
                    ui.selectable_value(
                        &mut self.controller_type,
                        CameraControllerType::Orbit,
                        "Orbit",
                    );
                });
            });
    }
}

pub struct ConstantSection {
    gravitational_constant: f64,
}
impl Default for ConstantSection {
    fn default() -> Self {
        Self {
            gravitational_constant: crate::util::BIG_G,
        }
    }
}
impl super::View for ConstantSection {
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Constants")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Gravitational Constant:");
                    ui.add(
                        egui::DragValue::new(&mut self.gravitational_constant)
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

pub struct TimeSection {
    time_scale: f64,
    current_date_time: DateTime<Local>,
}
impl Default for TimeSection {
    fn default() -> Self {
        Self {
            time_scale: 86400.0,
            current_date_time: Local::now(),
        }
    }
}
impl super::View for TimeSection {
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Time")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Time Scale:");
                    ui.add(
                        egui::Slider::new(&mut self.time_scale, 0.0..=3_155_760_000.0)
                            .logarithmic(true)
                            .custom_formatter(dynamic_exponent_formatter()),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Date:");

                    let mut date = self.current_date_time.date().with_timezone(&Utc);
                    ui.add(egui_extras::DatePickerButton::new(&mut date));
                    self.current_date_time = date
                        .with_timezone(&Local)
                        .and_time(self.current_date_time.time())
                        .unwrap();

                    ui.label("Time:");
                    ui.add(DateTimeValue::new(
                        "time_value",
                        &mut self.current_date_time,
                    ));
                });

                ui.horizontal(|ui| {
                    ui.label("Julian Date:");
                    // TODO: IMplement julian date
                    ui.label("TODO");
                });
            });
    }
}
