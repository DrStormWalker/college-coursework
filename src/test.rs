#![cfg(test)]

#[test]
fn test_keplerian_conversion() {
    use crate::simulation::util::keplerian_to_cartesian;
    use crate::util::BIG_G;

    // Values from [Wikipedia](https://en.wikipedia.org/wiki/Earth's_orbit#Events)
    let (_, earth_vel) = keplerian_to_cartesian(
        149.60e9,               // Semi-major axis
        0.0167086,              // Eccentricity
        288.1_f64.to_radians(), // Argument of periapsis
        174.9_f64.to_radians(), // Longitude of ascending node
        7.155_f64.to_radians(), // inclination of orbit
        2000.0,
        2000.0,
        171.1_f64.to_radians(),
        BIG_G * 1.9885e30,
    );

    assert!(false, "{}", earth_vel.magnitude());
}
