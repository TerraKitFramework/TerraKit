//! Coordinate hashing implementations.

use terrakit_core::GenerationSeed;

const SEED_OFFSET: u64 = 0x9E37_79B9_7F4A_7C15;
const X_SALT: u64 = 0xD1B5_4A32_D192_ED03;
const Y_SALT: u64 = 0xABC9_83A3_8B1B_DA8D;
const Z_SALT: u64 = 0x8CB9_2BA7_2F3D_8DD7;
const STEP_SALT: u64 = 0xDB4F_0B91_75AE_2165;

/// Hashes a 1D signed integer lattice coordinate with a deterministic seed.
///
/// Negative coordinates are supported. The result is stable across supported
/// platforms and is intended for procedural generation, not cryptography.
pub fn hash_1d(seed: GenerationSeed, x: i64) -> u64 {
    let mut value = seed.value().wrapping_add(SEED_OFFSET);
    value ^= mix_axis(x, X_SALT);
    mix_u64(value)
}

/// Hashes a 2D signed integer lattice coordinate with a deterministic seed.
///
/// Negative coordinates are supported, and coordinate ordering matters:
/// `(x, y)` is hashed differently from `(y, x)` for representative values.
/// The result is intended for procedural generation, not cryptography.
pub fn hash_2d(seed: GenerationSeed, x: i64, y: i64) -> u64 {
    let mut value = seed.value().wrapping_add(SEED_OFFSET);
    value ^= mix_axis(x, X_SALT);
    value = value.wrapping_mul(STEP_SALT).wrapping_add(Y_SALT);
    value ^= mix_axis(y, Y_SALT);
    mix_u64(value)
}

/// Hashes a 3D signed integer lattice coordinate with a deterministic seed.
///
/// Negative coordinates are supported, and X, Y, and Z positions use distinct
/// axis salts so coordinate ordering affects the result. The result is intended
/// for procedural generation, not cryptography.
pub fn hash_3d(seed: GenerationSeed, x: i64, y: i64, z: i64) -> u64 {
    let mut value = seed.value().wrapping_add(SEED_OFFSET);
    value ^= mix_axis(x, X_SALT);
    value = value.wrapping_mul(STEP_SALT).wrapping_add(Y_SALT);
    value ^= mix_axis(y, Y_SALT);
    value = value.wrapping_mul(STEP_SALT).wrapping_add(Z_SALT);
    value ^= mix_axis(z, Z_SALT);
    mix_u64(value)
}

/// Converts a hash value to an approximately normalized `0.0..=1.0` scalar.
///
/// The conversion is deterministic and performs no clamping.
pub fn hash_to_unit_f32(hash: u64) -> f32 {
    (hash as f64 / u64::MAX as f64) as f32
}

/// Converts a hash value to an approximately normalized `-1.0..=1.0` scalar.
///
/// The conversion is deterministic and performs no clamping.
pub fn hash_to_signed_f32(hash: u64) -> f32 {
    (hash_to_unit_f32(hash) * 2.0) - 1.0
}

fn mix_axis(coordinate: i64, salt: u64) -> u64 {
    mix_u64((coordinate as u64).wrapping_add(salt))
}

fn mix_u64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}
