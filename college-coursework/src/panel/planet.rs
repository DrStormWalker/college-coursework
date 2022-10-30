use cgmath::Vector3;
use specs::{Component, VecStorage};

use crate::simulation::Identifier;

use super::{dynamic_exponent_formatter, global::MINUS_ONE_EXPONENT, Vector3Value};

#[derive(Component)]
#[storage(VecStorage)]
pub struct PlanetWindowShown(pub bool);
impl Default for PlanetWindowShown {
    fn default() -> Self {
        Self(false)
    }
}

pub struct PlanetWindow<'a> {
    pub id: Identifier,
    pub position: &'a mut Vector3<f64>,
    pub velociy: &'a mut Vector3<f64>,
    pub mass: &'a mut f64,
}
impl<'a> PlanetWindow<'a> {
    pub fn get_id(&self) -> Identifier {
        self.id.clone()
    }
}
impl<'a> super::Window for PlanetWindow<'a> {
    fn name(&self) -> &str {
        self.id.get_name()
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
impl<'a> super::View for PlanetWindow<'a> {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.label(self.id.get_name());
            ui.label("ID:");
            ui.label(self.id.get_id());
        });

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
            ui.label("Velocity:");
            ui.add(
                Vector3Value::new(self.velociy)
                    .custom_formatter(dynamic_exponent_formatter())
                    .suffix(const_format::concatcp!(" ms", MINUS_ONE_EXPONENT))
                    .speed(0.1),
            )
        });

        ui.horizontal(|ui| {
            ui.label("Mass:");
            ui.add(
                egui::DragValue::new(self.mass)
                    .speed(0.1)
                    .custom_formatter(dynamic_exponent_formatter()),
            )
        });
    }
}
