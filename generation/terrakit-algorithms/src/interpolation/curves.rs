//! Interpolation helpers and easing curves.

/// Linearly interpolates between `a` and `b` using `t`.
///
/// `t` is normally in `0.0..=1.0`, but this function deliberately does not
/// clamp it so callers can use extrapolation when desired.
pub fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + ((b - a) * t)
}

/// Linearly interpolates between `a` and `b` using `t`.
///
/// `t` is normally in `0.0..=1.0`, but this function deliberately does not
/// clamp it so callers can use extrapolation when desired.
pub fn lerp_f64(a: f64, b: f64, t: f64) -> f64 {
    a + ((b - a) * t)
}

/// Computes the cubic smoothstep curve `t * t * (3 - 2 * t)`.
///
/// `t` is expected to be in `0.0..=1.0`. This function does not clamp.
pub fn smoothstep(t: f64) -> f64 {
    t * t * (3.0 - (2.0 * t))
}

/// Computes the quintic smootherstep curve.
///
/// `t` is expected to be in `0.0..=1.0`. This function does not clamp.
pub fn smootherstep(t: f64) -> f64 {
    t * t * t * (t * ((t * 6.0) - 15.0) + 10.0)
}

/// Clamps `t` to `0.0..=1.0` and computes [`smoothstep`].
pub fn smoothstep_clamped(t: f64) -> f64 {
    smoothstep(t.clamp(0.0, 1.0))
}

/// Clamps `t` to `0.0..=1.0` and computes [`smootherstep`].
pub fn smootherstep_clamped(t: f64) -> f64 {
    smootherstep(t.clamp(0.0, 1.0))
}
