import math
import numpy as np

def keplerian_to_cartesian(
    a: float,
    e: float,
    w: float,
    omega: float,
    i: float,
    t0: float,
    t: float,
    M0: float,
    mu: float,
) -> (np.array, np.array):
    """
    Converts Keplerian Orbit Elemants into Cartesian State Vectors,
    Written using the algorithm described in:
    - https://downloads.rene-schwarz.com/download/M001-Keplerian_Orbit_Elements_to_Cartesian_State_Vectors.pdf
    With the help of the guide:
    - https://devforum.roblox.com/t/converting-keplerian-orbit-elements-to-cartesian-state-vectors/1705863
    
    :param float a: semi-major axis in meters
    :param float e: eccentricity of orbit
    :param float w: (lower case omega) argument of periapsis (in radians)
    :param float omega: Longitude of ascending node (in radians)
    :param float i: inclination (in radians)
    :param float t0: epoch of mean anomaly (Julian date)
    :param float t: considered epoch (Julian date) different to t0
    :param float M0: Mean anomaly (in radians) at epoch t0
    :param float mu: standard gravitational parameter mu = G * M,
        (G is the universal gravitational constant, M is the central body mass)
    
    :return: The cartesian state vectors for the orbit (position, velocity)
    """

    if t == t0:
        Mt = M0
    else:
        # Calculate the time difference, converting from Julian dates to seconds
        dt = 86400 * (t - t0)
        
        # Calculate the mean anomaly
        Mt = M0 + dt * math.sqrt(mu / a ** 3)

    # Solve Kepler's equation M(t) = E(t) - e * sin(E) for the anomaly E(T)
    # using the Newton-Raphson method
    solveIterations = 30
    i = 0
    E = Mt
    F = E - e * math.sin(E) - Mt
    
    # Newton-Raphson iterations
    while i < solveIterations:
        E = E - F / (1 - e * math.cos(E))
        F = E - e * math.sin(E) - Mt
        
        i += 1

    # Obtain the true anomaly nu(t)
    nu = 2 * math.atan2(
        math.sqrt(1 + e) * math.sin(E / 2),
        math.sqrt(1 - e) * math.cos(E / 2),
    )

    # Use the eccentric anomaly E to get the distance to the central body
    rc = a * (1 - e * math.cos(E))

    # Obtain the position vector o in the orbital frame
    # The z-axis is perpendicular to the orbital frame
    # The x-axis is pointing to the periapsis of the orbit
    o = rc * np.array([math.cos(nu), maths.sin(nu), 0])
    
    # obtain the velocity vector odot in the orbital fram
    # x and z axies are the same as above
    odot = math.sqrt(mu * a) / rc * np.array([
        -math.sin(E), math.sqrt(1 - e*e) * math.cos(E), 0
    ])

    # Transform o and odot to the inertial frame
    r = np.array([
        o[0] * (math.cos(w) * math.cos(omega) - math.sin(w) * math.cos(i) * math.sin(omega))
            - o[1] * (math.sin(w) * math.cos(omega) + math.cos(w) * math.cos(i) * math.sin(omega)),
        o[0] * (math.cos(w) * math.sin(omega) + math.sin(w) * math.cos(i) * math.cos(omega))
            - o[1] * (math.cos(w) * math.cos(i) * math.cos(omega) - math.sin(w) * math.sin(omega)),
        o[0] * (math.sin(w) * math.sin(i)) + o[1] * (math.cos(w) * math.sin(i)),
    ])

    rdot = np.array([
        odot[0] * (math.cos(w) * math.cos(omega) - math.sin(w) * math.cos(i) * math.sin(omega))
            - odot[1] * (math.sin(w) * math.cos(omega) + math.cos(w) * math.cos(i) * math.sin(omega)),
        odot[0] * (math.cos(w) * math.sin(omega) + math.sin(w) * math.cos(i) * math.cos(omega))
            - odot[1] * (math.cos(w) * math.cos(i) * math.cos(omega) - math.sin(w) * math.sin(omega)),
        odot[0] * (math.sin(w) * math.sin(i)) + odot[1] * (math.cos(w) * math.sin(i)),
    ])

    return r, rdot

keplerian_to_cartesian()

