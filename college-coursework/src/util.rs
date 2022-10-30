use chrono::{Date, DateTime, Duration, Utc};
use nalgebra::{Vector2, Vector3};

pub const BIG_G: f64 = 6.6743015e-11;
pub const AU: f64 = 1.495978707e11;

pub type Vec2 = Vector2<f64>;
pub type Vec3 = Vector3<f64>;

#[rustfmt::skip]
pub fn convert_date_to_julian_day(date: &Date<Utc>) -> i64 {
    use chrono::Datelike as _;
    (1461 * (date.year() as i64 + 4800 + (date.month() as i64 - 14) / 12)) / 4
        + (367 * (date.month() as i64 - 2 - 12 * ((date.month() as i64 - 14) / 12))) / 12
        - (3 * ((date.year() as i64 + 4900 + (date.month() as i64 - 14) / 12) / 100)) / 4
        + date.day() as i64
        - 32075
}

pub fn convert_julian_day_to_date(day: i64) -> Date<Utc> {
    // Algorithm parameters for gregorian calendar
    let y = 4716;
    let j = 1401;
    let m = 2;
    let n = 12;
    let r = 4;
    let p = 1461;
    let v = 3;
    let u = 5;
    let s = 153;
    let w = 2;
    let b = 274277;
    let c = -38;

    let f = day + j + (((4 * day + b) / 146097) * 3) / 4 + c;
    let e = r * f + v;
    let g = (e % p) / r;
    let h = u * g + w;
    let day = (h % s) / u + 1;
    let month = (h / s + m) % n + 1;
    let year = e / p - y + (n + m - month) / n;

    use chrono::TimeZone as _;
    Utc.ymd(year as i32, month as u32, day as u32)
}

#[rustfmt::skip]
pub fn convert_datetime_to_julian_date(datetime: &DateTime<Utc>) -> f64 {
    use chrono::Timelike as _;
    let julian_day_number = convert_date_to_julian_day(&datetime.date());
    let mut date = julian_day_number as f64
        + (datetime.hour() as f64 - 12.0) / 24.0
        + datetime.minute() as f64 / 1440.0
        + datetime.second() as f64 / 86400.0;
    
    if datetime.hour() >= 12 && datetime.hour() <= 23 {
        date += 1.0;
    }
    
    date
}

pub fn convert_julian_date_to_datetime(julian_date: f64) -> DateTime<Utc> {
    let time = julian_date.fract();
    let mut date = convert_julian_day_to_date(julian_date as i64);

    let mut hour = (time * 24.0 + 12.0) as u32;
    let time = time - (hour as f64 - 12.0) / 24.0;
    let minute = (time * 1440.0) as u32;
    let time = time - minute as f64 / 1440.0;
    let mut second = time * 86400.0;

    if second.floor() + 1.0 - second < 1e-3 && second.round() < 60.0 {
        second += 1.0;
    }

    if hour >= 24 {
        date += Duration::days(1);
        hour = hour - 24;
    }

    date.and_hms(hour, minute, second as u32)
}

mod tests {
    use crate::util::{
        convert_date_to_julian_day, convert_datetime_to_julian_date,
        convert_julian_date_to_datetime, convert_julian_day_to_date,
    };

    #[test]
    fn test_julian_day_conversion() {
        use chrono::{TimeZone as _, Utc};
        let date = Utc.ymd(2022, 10, 15);
        let julian_day = convert_date_to_julian_day(&date);
        assert_eq!(2459868, julian_day);
        assert_eq!(date, convert_julian_day_to_date(julian_day));
    }

    #[test]
    fn test_julian_date_conversion() {
        use chrono::{TimeZone as _, Utc};
        let datetime = Utc.ymd(2000, 01, 01).and_hms(18, 0, 0);
        let julian_datetime = convert_datetime_to_julian_date(&datetime);
        assert_eq!(2451545.25, julian_datetime);
        assert_eq!(datetime, convert_julian_date_to_datetime(julian_datetime));

        let datetime = Utc.ymd(2022, 10, 15).and_hms(15, 05, 28);
        let julian_datetime = convert_datetime_to_julian_date(&datetime);
        assert_eq!(2459868.128796296, julian_datetime);
        assert_eq!(datetime, convert_julian_date_to_datetime(julian_datetime));
    }
}
