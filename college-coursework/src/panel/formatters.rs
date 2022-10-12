use std::ops::RangeInclusive;

use egui::emath::Numeric;

pub fn dynamic_exponent_formatter() -> impl Fn(f64, RangeInclusive<usize>) -> String {
    |value: f64, _| {
        let log10 = value.abs().log10();
        if log10 > -4.0 && log10 < 6.0 || value == 0.0 {
            format!("{}", value)
        } else {
            format!("{:e}", value)
        }
    }
}
