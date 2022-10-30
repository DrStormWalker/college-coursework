use std::ops::RangeInclusive;

use egui::emath::Numeric;

pub fn dynamic_exponent_formatter() -> impl Fn(f64, RangeInclusive<usize>) -> String {
    |value: f64, _| {
        let log10 = value.abs().log10();
        if log10 > -4.0 && log10 < 6.0 || value == 0.0 {
            format!("{:.3}", value)
        } else {
            format!("{:.6e}", value)
        }
    }
}

pub fn dynamic_decimals_formatter() -> impl Fn(f64, RangeInclusive<usize>) -> String {
    |value: f64, _| format!("{}", value)
}
