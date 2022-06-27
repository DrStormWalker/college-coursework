use crate::util::Vec3;

/// Returns Cartesian State Vectors converted from the given Keplerian
/// Orbital Elements
///
/// Based on the algorithm outline in
/// [this](https://downloads.rene-schwarz.com/download/M001-Keplerian_Orbit_Elements_to_Cartesian_State_Vectors.pdf)
/// paper by René Schwarz.
///
/// # Arguments
///
/// * `a` - Semi-major axis (in meters)
/// * `e` - The eccentricity of the orbit
/// * `w` - (representing lower case omega) Argument of periapsis (in radians)
/// * `omega` - Longitude of ascending node (in radians)
/// * `i` - inclination of orbit (in radians)
/// * `t0` - Epoch of mean anomaly (in Julian date)
/// * `t` - Considered epoch (in Julian date)
/// * `m0` - Mean anomaly at epoch `t0` (in radians)
/// * `mu` - Standard gravitational parameter, mu = G * M where G is the Universal Gravitational
///          Constant and M is the central body mass
///
pub fn keplerian_to_cartesian(
    a: f64,
    e: f64,
    w: f64,
    omega: f64,
    i: f64,
    t0: f64,
    t: f64,
    m0: f64,
    mu: f64,
) -> (Vec3, Vec3) {
    let mt = if t == t0 {
        m0
    } else {
        // Calculate the time difference, converting from Julian date to seconds
        let dt = 86400.0 * (t - t0);

        // Calculate the mean anomaly
        m0 + dt * (mu / a.powi(3)).sqrt()
    };

    // Solve Kepler's equation M(t) = E(t) - e * sin(E) for the eccentric anomaly `big_e`
    // using the Netwon-Raphson method

    // Maximum iterations
    const NEWTON_RAPHSON_ITERATIONS: usize = 30;
    // Error margin
    const ERROR_MARGIN: f64 = 0.0001;

    // Initialise variables
    let mut j = 0;
    let mut big_e = mt;
    let mut big_f = big_e - e * big_e.sin() - mt;

    // Perform Newton-Raphson iterations, exiting after a maximum number of iterations
    // or if the error margin becomes small enough
    while j < NEWTON_RAPHSON_ITERATIONS {
        big_e -= big_f / (1.0 - e * big_e.cos());
        big_f = big_e - e * big_e.sin() - mt;

        j += 1
    }

    // Calculate the true anomaly `nu`
    let nu = 2.0
        * f64::atan2(
            (1.0 + e).sqrt() * (big_e / 2.0).sin(),
            (1.0 - e).sqrt() * (big_e / 2.0).cos(),
        );

    // Use the eccentric anomaly `big_e` to get the distance to the central body
    let rc = a * (1.0 - e * big_e.cos());

    // Obtain the position vector `o` in the oribtal frame
    // - The z-axis is perpendicular to the orbital frame
    // - The x-axis is pointing to the periapsis of the orbit
    let o = rc * Vec3::new(nu.cos(), nu.sin(), 0.0);

    // Obtain the velocity vector `o_dot` in the same orbital frame
    // - The x and z axis are the same as above
    let o_dot =
        (mu * a).sqrt() / rc * Vec3::new(-big_e.sin(), (1.0 - e * e).sqrt() * big_e.cos(), 0.0);

    // Transform `o` and `o_dot` to the inertial frame in bodycentric regular
    // coordinates `r` and `r_dot`
    let r = Vec3::new(
        o.x * (w.cos() * omega.cos() - w.sin() * i.cos() * omega.sin())
            - o.y * (w.sin() * omega.cos() + w.cos() * i.cos() * omega.sin()),
        o.x * (w.cos() * omega.sin() + w.sin() * i.cos() * omega.cos())
            + o.y * (w.cos() * i.cos() * omega.cos() - w.sin() * omega.sin()),
        o.x * (w.sin() * i.sin()) + o.y * (w.cos() * i.sin()),
    );

    let r_dot = Vec3::new(
        o_dot.x * (w.cos() * omega.cos() - w.sin() * i.cos() * omega.sin())
            - o_dot.y * (w.sin() * omega.cos() + w.cos() * i.cos() * omega.sin()),
        o_dot.x * (w.cos() * omega.sin() + w.sin() * i.cos() * omega.cos())
            + o_dot.y * (w.cos() * i.cos() * omega.cos() - w.sin() * omega.sin()),
        o_dot.x * (w.sin() * i.sin()) + o_dot.y * (w.cos() * i.sin()),
    );

    // Return the resulting cartesian state vectors
    (r, r_dot)
}
