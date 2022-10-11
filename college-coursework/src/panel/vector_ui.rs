pub fn input_vector3<S: egui::emath::Numeric>(
    value: &mut [S; 3],
    unit: impl ToString,
) -> impl egui::Widget + '_ {
    let unit = unit.to_string();
    move |ui: &mut egui::Ui| {
        ui.columns(3, |cols| {
            cols.iter_mut()
                .zip(["x", "y", "z"].into_iter())
                .zip(value.iter_mut())
                .map(|((ui, component), value)| {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", component));
                        let size = ui.available_size();
                        ui.add_sized(
                            size,
                            egui::DragValue::new(value)
                                .speed(0.1)
                                .custom_formatter(|n, _| {
                                    if n.log10() > -4.0 && n.log10() < 8.0 || n == 0.0 {
                                        format!("{:.}", n)
                                    } else {
                                        format!("{:e}", n)
                                    }
                                })
                                .suffix(&unit),
                        );
                    })
                    .response
                })
                .reduce(|a, b| a.union(b))
                .unwrap()
        })
    }
}
