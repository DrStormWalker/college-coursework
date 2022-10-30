use std::{fmt::Display, ops::RangeInclusive};

use cgmath::Vector3;
use chrono::{DateTime, Duration, NaiveDateTime, NaiveTime, TimeZone};
use egui::{emath::Numeric, Widget};

type NumFormatter<'a> = Box<dyn 'a + Fn(f64, RangeInclusive<usize>) -> String>;

pub struct Vector3Value<'a, S: Numeric> {
    value: &'a mut Vector3<S>,
    prefix: String,
    suffix: String,
    speed: f64,
    formatter: Option<NumFormatter<'a>>,
    labels: bool,
}
impl<'a, S: Numeric> Vector3Value<'a, S> {
    pub fn new(value: &'a mut Vector3<S>) -> Self {
        Self {
            value,
            prefix: String::new(),
            suffix: String::new(),
            speed: 1.0,
            formatter: None,
            labels: true,
        }
    }

    pub fn prefix(mut self, prefix: impl ToString) -> Self {
        self.prefix = prefix.to_string();
        self
    }

    pub fn suffix(mut self, suffix: impl ToString) -> Self {
        self.suffix = suffix.to_string();
        self
    }

    pub fn speed(mut self, speed: impl Into<f64>) -> Self {
        self.speed = speed.into();
        self
    }

    pub fn custom_formatter(
        mut self,
        formatter: impl 'a + Fn(f64, RangeInclusive<usize>) -> String,
    ) -> Self {
        self.formatter = Some(Box::new(formatter));
        self
    }

    pub fn labels(mut self, labels: bool) -> Self {
        self.labels = labels;
        self
    }
}
impl<'a, S: Numeric> Widget for Vector3Value<'a, S> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut values: [S; 3] = (*self.value).into();
        let response = ui.columns(3, |cols| {
            cols.iter_mut()
                .zip(["x", "y", "z"].into_iter())
                .zip(values.iter_mut())
                .map(|((ui, component), value)| {
                    ui.horizontal(|ui| {
                        if self.labels {
                            ui.label(format!("{}:", component));
                        }

                        let size = ui.available_size();
                        let drag_value = egui::DragValue::new(value)
                            .speed(self.speed)
                            .prefix(&self.prefix)
                            .suffix(&self.suffix);

                        let drag_value = if let Some(formatter) = self.formatter.as_ref() {
                            drag_value.custom_formatter(|v, r| formatter(v, r))
                        } else {
                            drag_value
                        };

                        ui.add_sized(size, drag_value);
                    })
                    .response
                })
                .reduce(|a, b| a.union(b))
                .unwrap()
        });

        *self.value = values.into();

        response
    }
}

pub struct DateTimeValue<'a, Tz: TimeZone> {
    id: egui::Id,
    date_time: &'a mut DateTime<Tz>,
    format: &'a str,
    speed: f64,
}
impl<'a, Tz: TimeZone> DateTimeValue<'a, Tz> {
    pub fn new(id: impl std::hash::Hash, date_time: &'a mut DateTime<Tz>) -> Self {
        Self {
            id: egui::Id::new(id),
            date_time,
            format: "%H:%M:%S",
            speed: 1.0,
        }
    }
}
impl<'a, Tz: TimeZone> Widget for DateTimeValue<'a, Tz>
where
    Tz::Offset: Display,
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let Self {
            id,
            date_time,
            format,
            speed,
        } = self;

        let shift = ui.input().modifiers.shift_only();
        let is_slow_speed = shift && ui.memory().is_being_dragged(self.id);

        let old_value = date_time.clone();
        let mut value_text = date_time.format(format).to_string();

        let kb_edit_id = id.with(0);
        let is_kb_editing = ui.memory().has_focus(kb_edit_id);

        let mut response = if is_kb_editing {
            let button_width = ui.spacing().interact_size.x;
            let response = ui.add(
                egui::TextEdit::singleline(&mut value_text)
                    .id(kb_edit_id)
                    .desired_width(button_width)
                    .font(egui::TextStyle::Monospace),
            );

            if let Ok(parsed_value) = NaiveDateTime::parse_from_str(&value_text, format)
                .map(|dt| dt.and_local_timezone(date_time.timezone()).unwrap())
            {
                *date_time = parsed_value;
            }

            if ui.input().key_pressed(egui::Key::Enter) {
                ui.memory().surrender_focus(kb_edit_id);
            }
            response
        } else {
            let button = egui::Button::new(egui::RichText::new(&value_text).monospace())
                .wrap(false)
                .sense(egui::Sense::click_and_drag());

            let response = ui.add_sized(ui.spacing().interact_size, button);

            let mut response = response.on_hover_cursor(egui::CursorIcon::ResizeHorizontal);

            if ui.style().explanation_tooltips {
                response = response.on_hover_text(format!(
                    "{}\nDrag to edit or click to enter a value.\nPress 'Shift' while dragging for better control.",
                    &value_text,
                ));
            }

            if response.clicked() {
                ui.memory().request_focus(kb_edit_id);
            } else if response.dragged() {
                ui.output().cursor_icon = egui::CursorIcon::ResizeHorizontal;

                let mdelta = response.drag_delta();
                let delta_points = mdelta.x - mdelta.y;

                let speed = if is_slow_speed { speed / 10.0 } else { speed };

                let delta_value = delta_points as f64 * speed;

                if delta_value != 0.0 {
                    let delta_duration =
                        Duration::from_std(std::time::Duration::from_secs_f64(delta_value.abs()))
                            .unwrap();

                    if delta_value > 0.0 {
                        *date_time += delta_duration;
                    } else {
                        *date_time -= delta_duration;
                    }
                }
            } else if response.has_focus() {
                let change = ui.input().num_presses(egui::Key::ArrowUp) as f64
                    + ui.input().num_presses(egui::Key::ArrowRight) as f64
                    - ui.input().num_presses(egui::Key::ArrowDown) as f64
                    - ui.input().num_presses(egui::Key::ArrowLeft) as f64;

                let delta_value = change * speed;

                if change != 0.0 {
                    let delta_duration =
                        Duration::from_std(std::time::Duration::from_secs_f64(delta_value.abs()))
                            .unwrap();

                    if delta_value > 0.0 {
                        *date_time += delta_duration;
                    } else {
                        *date_time -= delta_duration;
                    }
                }
            }

            response
        };
        response.changed = *date_time != old_value;

        response
    }
}
