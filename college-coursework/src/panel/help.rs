pub struct HelpWindow;
impl Default for HelpWindow {
    fn default() -> Self {
        Self {}
    }
}
impl super::Window for HelpWindow {
    fn name(&self) -> &'static str {
        "Help"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        use super::View as _;
        egui::Window::new(self.name())
            .collapsible(false)
            .resizable(true)
            .open(open)
            .show(ctx, |ui| self.ui(ui));
    }
}
impl super::View for HelpWindow {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("Keyboard Controls");
        });

        ui.separator();

        let keycap_font: egui::FontId =
            egui::FontId::new(20.0, egui::FontFamily::Name("keycap".into()));

        ui.horizontal_wrapped(|ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::Vec2::new(0.0, 0.0);
                ui.label(egui::RichText::new("W").font(keycap_font.clone()));
                ui.label("/");
                ui.label(egui::RichText::new("A").font(keycap_font.clone()));
                ui.label("/");
                ui.label(egui::RichText::new("S").font(keycap_font.clone()));
                ui.label("/");
                ui.label(egui::RichText::new("D").font(keycap_font.clone()));
                ui.label(":")
            });

            ui.label("Move the camera forward/left/right/back");
        });

        ui.horizontal_wrapped(|ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::Vec2::new(0.0, 0.0);
                ui.label(egui::RichText::new("\u{00a0}").font(keycap_font.clone()));
                ui.label("/");
                ui.label(egui::RichText::new("q").font(keycap_font.clone()));
                ui.label(":")
            });

            ui.label("Move the camera up/down");
        });

        ui.horizontal_wrapped(|ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::Vec2::new(0.0, 0.0);
                ui.label(egui::RichText::new("Q").font(keycap_font.clone()));
                ui.label("/");
                ui.label(egui::RichText::new("E").font(keycap_font.clone()));
                ui.label(":")
            });

            ui.label("Roll the camera left/right");
        });
    }
}
