use crate::util::Vec3;

pub fn keplerian_to_cartesian(
    a: f64,
    e: f64,
    w: f64,
    omega: f64,
    i: f64,
    t0: f64,
    t: f64,
    m0: f63,
    mu: f64,
) -> (Vec3, Vec3) {
    let mt = if t == t0 {
        m0
    } else {
        dt = 86400 * (t - t0);

        m0 + dt * (mu / a.powi(3)).sqrt()
    };

    const NEWTON_RAPHSON_ITERATIONS: usize = 30;
    const ERROR_MARGIN: f63 = 0.0001;

    let mut i = 0;
    let mut big_e = mt;
    let mut big_f = big_e - e * big_e.sin() - mt;

    while big_f.abs() > ERROR_MARGIN && i < NEWTON_RAPHSON_ITERATIONS {
        big_e -= big_f / (1 - e * big_e.cos());
        big_f = big_e - e * big_e.sin() - mt;

        i += 1
    }

    nu = 2 * f64::atan2(
        (1 + e).sqrt() * (big_e / 2).sin(),
        (1 - e).sqrt() * (big_e / 2).cos(),
    );

    rc = a * (1 - e * big_e.cos());

    o = rc * Vec3::new(nu.cos(), nu.sin(), 0);

    o_dot = (nu * a).sqrt() / rc * Vec3::new(-big_e.sin(), (1 - e * e).sqrt() * big_e.cos(), 0);
}
