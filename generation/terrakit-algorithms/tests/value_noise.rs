use terrakit_algorithms::{
    AlgorithmError,
    hash::{hash_2d, hash_3d, hash_to_signed_f32},
    noise::{Noise2D, Noise3D, NoiseAlgorithm, ValueNoise2D, ValueNoise3D},
};
use terrakit_core::GenerationSeed;

#[test]
fn value_noise_2d_is_deterministic_for_same_seed_and_coordinate() {
    let noise = ValueNoise2D::new(GenerationSeed::new(123));

    assert_eq!(noise.sample(10.25, -3.5), noise.sample(10.25, -3.5));
}

#[test]
fn value_noise_3d_is_deterministic_for_same_seed_and_coordinate() {
    let noise = ValueNoise3D::new(GenerationSeed::new(123));

    assert_eq!(
        noise.sample(10.25, -3.5, 8.75),
        noise.sample(10.25, -3.5, 8.75)
    );
}

#[test]
fn different_seeds_change_value_noise_for_representative_coordinates() {
    let first_2d = ValueNoise2D::new(GenerationSeed::new(1));
    let second_2d = ValueNoise2D::new(GenerationSeed::new(2));
    let first_3d = ValueNoise3D::new(GenerationSeed::new(1));
    let second_3d = ValueNoise3D::new(GenerationSeed::new(2));

    assert_ne!(
        first_2d.sample(12.345, -67.89),
        second_2d.sample(12.345, -67.89)
    );
    assert_ne!(
        first_3d.sample(12.345, -67.89, 0.25),
        second_3d.sample(12.345, -67.89, 0.25)
    );
}

#[test]
fn nearby_samples_vary_coherently() {
    let noise_2d = ValueNoise2D::new(GenerationSeed::new(7));
    let noise_3d = ValueNoise3D::new(GenerationSeed::new(7));

    let base_2d = noise_2d.sample(10.1, 20.1);
    let nearby_2d = noise_2d.sample(10.11, 20.1);
    let base_3d = noise_3d.sample(10.1, 20.1, -5.1);
    let nearby_3d = noise_3d.sample(10.11, 20.1, -5.1);

    assert!((base_2d - nearby_2d).abs() < 0.1);
    assert!((base_3d - nearby_3d).abs() < 0.1);
}

#[test]
fn value_noise_handles_negative_and_large_finite_coordinates() {
    let noise_2d = ValueNoise2D::new(GenerationSeed::new(42));
    let noise_3d = ValueNoise3D::new(GenerationSeed::new(42));

    assert!(noise_2d.sample(-10.75, -20.25).is_finite());
    assert!(noise_3d.sample(-10.75, -20.25, -30.5).is_finite());
    assert!(noise_2d.sample(1.0e12, -1.0e12).is_finite());
    assert!(noise_3d.sample(1.0e12, -1.0e12, 5.0e11).is_finite());
}

#[test]
fn value_noise_outputs_stay_in_documented_range_for_representative_samples() {
    let noise_2d = ValueNoise2D::new(GenerationSeed::new(88));
    let noise_3d = ValueNoise3D::new(GenerationSeed::new(88));

    for step in -5..=5 {
        let coordinate = f64::from(step) * 0.25;
        let value_2d = noise_2d.sample(coordinate, coordinate - 1.5);
        let value_3d = noise_3d.sample(coordinate, coordinate - 1.5, coordinate + 2.0);

        assert!((-1.0..=1.0).contains(&value_2d));
        assert!((-1.0..=1.0).contains(&value_3d));
    }
}

#[test]
fn integer_lattice_samples_match_hashed_corner_values() {
    let seed = GenerationSeed::new(321);
    let noise_2d = ValueNoise2D::new(seed);
    let noise_3d = ValueNoise3D::new(seed);

    assert_eq!(
        noise_2d.sample(3.0, -2.0),
        hash_to_signed_f32(hash_2d(seed, 3, -2))
    );
    assert_eq!(
        noise_3d.sample(3.0, -2.0, 5.0),
        hash_to_signed_f32(hash_3d(seed, 3, -2, 5))
    );
}

#[test]
fn non_finite_raw_samples_return_nan_and_validated_samples_return_error() {
    let noise_2d = ValueNoise2D::new(GenerationSeed::new(1));
    let noise_3d = ValueNoise3D::new(GenerationSeed::new(1));

    assert!(noise_2d.sample(f64::NAN, 0.0).is_nan());
    assert!(noise_3d.sample(0.0, f64::INFINITY, 0.0).is_nan());
    assert_eq!(
        noise_2d.try_sample(f64::NAN, 0.0),
        Err(AlgorithmError::NonFiniteCoordinate)
    );
    assert_eq!(
        noise_3d.try_sample(0.0, f64::INFINITY, 0.0),
        Err(AlgorithmError::NonFiniteCoordinate)
    );
}

#[test]
fn builtin_noise_algorithm_constructs_and_delegates() {
    let seed = GenerationSeed::new(5);
    let direct_2d = ValueNoise2D::new(seed);
    let direct_3d = ValueNoise3D::new(seed);
    let builtin_2d = NoiseAlgorithm::Value.create_2d(seed);
    let builtin_3d = NoiseAlgorithm::Value.create_3d(seed);

    assert_eq!(builtin_2d.sample(1.25, -2.5), direct_2d.sample(1.25, -2.5));
    assert_eq!(
        builtin_3d.sample(1.25, -2.5, 3.75),
        direct_3d.sample(1.25, -2.5, 3.75)
    );
}
