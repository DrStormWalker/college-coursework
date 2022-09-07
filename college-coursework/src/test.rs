#![cfg(test)]

use cgmath::InnerSpace;

#[test]
fn test_keplerian_conversion_earth() {
    use crate::simulation::util::keplerian_to_cartesian;
    use crate::util::BIG_G;

    // Values from [Wikipedia](https://en.wikipedia.org/wiki/Earth's_orbit#Events)
    let (earth_pos, earth_vel) = keplerian_to_cartesian(
        149.60e9,               // Semi-major axis
        0.0167086,              // Eccentricity
        288.1_f64.to_radians(), // Argument of periapsis
        174.9_f64.to_radians(), // Longitude of ascending node
        7.155_f64.to_radians(), // inclination of orbit
        2000.0,                 // Epoch
        2000.0,
        171.1_f64.to_radians(), // Mean anomaly
        BIG_G * 1.9885e30,      // Standard gravitational parameter
    );

    // The distance of the Earth from the Sun is within 30 Gm
    assert!(
        (earth_pos.magnitude() - 152.098_455e9).abs() < 30e6,
        "Expected a distance of {}m but got {}m",
        152.098_455e9,
        earth_pos.magnitude() - 152.098_455e9,
    );

    // The speed is earth is within 10 meters of their actual value (at epoch 2000)
    assert!(
        (earth_vel.magnitude() - 29.29e3).abs() < 10.0,
        "Expected a speed of {}ms^-1 but got {}ms^-1",
        29.29e3,
        earth_vel.magnitude(),
    );
}

#[test]
fn test_keplerian_conversion_moon() {
    use crate::simulation::util::keplerian_to_cartesian;
    use crate::util::BIG_G;

    // Values from [Nasa](https://ssd.jpl.nasa.gov/sats/elem/)
    let (moon_pos, moon_vel) = keplerian_to_cartesian(
        384.748e6,               // Semi-major axis
        0.0549006,               // Eccentricity
        318.15_f64.to_radians(), // Argument of periapsis
        125.08_f64.to_radians(), // Longitude of ascending node
        6.4_f64.to_radians(),    // inclination of orbit
        2000.0,                  // Epoch
        2000.0,
        135.27_f64.to_radians(), // Mean anomaly
        BIG_G * 5.972e24,        // Standard gravitational parameter
    );

    // The distance of the Moon from the Earth is within 4 Mm
    assert!(
        (moon_pos.magnitude() - 404.132e6).abs() < 4e6,
        "Expected a distance of {}m but got {}m",
        404.132e6,
        moon_pos.magnitude(),
    );

    // The speed of the Moon is within 10 meters of their actual value (at epoch 2000)
    assert!(
        (moon_vel.magnitude() - 0.970e3).abs() < 10.0,
        "Expected a speed of {}ms^-1 but got {}ms^-1",
        1.022e3,
        moon_vel.magnitude(),
    );
}
