use super::input_vector3;

pub struct GlobalWindow {
    position: [f64; 3],
}
impl Default for GlobalWindow {
    fn default() -> Self {
        Self { position: [0.0; 3] }
    }
}
impl super::View for GlobalWindow {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Position: ");
            ui.add(input_vector3(&mut self.position, " m"))
        });
    }
}
impl super::Window for GlobalWindow {
    fn name(&self) -> &'static str {
        "Global State"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        use super::View as _;
        egui::Window::new(self.name())
            .collapsible(false)
            .resizable(true)
            .default_height(500.0)
            .open(open)
            .show(ctx, |ui| self.ui(ui));
    }
}
