use terrakit_algorithms::{
    AlgorithmError,
    hash::{hash_1d, hash_2d, hash_3d, hash_to_signed_f32, hash_to_unit_f32},
    interpolation::{lerp_f32, lerp_f64, smootherstep, smoothstep},
    noise::{
        FractalSettings, Noise2D, Noise3D, ValueNoise2D, ValueNoise3D, fractal_sample_2d,
        fractal_sample_3d,
    },
};
use terrakit_core::{GenerationSeed, SeedDomain};

#[test]
fn seed_derivation_is_deterministic_and_domain_specific() {
    let seed = GenerationSeed::new(123);
    let domain = SeedDomain::new(7);

    assert_eq!(seed.derive(domain), seed.derive(domain));
    assert_ne!(seed.derive(domain), seed.derive(SeedDomain::new(8)));
    assert_eq!(seed, GenerationSeed::new(123));
}

#[test]
fn seed_derivation_supports_zero_and_max_values() {
    assert_eq!(
        GenerationSeed::new(0).derive(SeedDomain::new(0)),
        GenerationSeed::new(0).derive(SeedDomain::new(0))
    );
    assert_eq!(
        GenerationSeed::new(u64::MAX).derive(SeedDomain::new(u64::MAX)),
        GenerationSeed::new(u64::MAX).derive(SeedDomain::new(u64::MAX))
    );
}

#[test]
fn coordinate_hashing_is_stable_and_ordered() {
    let seed = GenerationSeed::new(42);

    assert_eq!(hash_1d(seed, -17), hash_1d(seed, -17));
    assert_eq!(hash_2d(seed, -17, 5), hash_2d(seed, -17, 5));
    assert_eq!(hash_3d(seed, -17, 5, -9), hash_3d(seed, -17, 5, -9));

    assert_ne!(hash_2d(seed, 1, 2), hash_2d(seed, 2, 1));
    assert_ne!(hash_3d(seed, 1, 2, 3), hash_3d(seed, 3, 2, 1));
    assert_ne!(
        hash_2d(seed, 11, 13),
        hash_2d(GenerationSeed::new(43), 11, 13)
    );
}

#[test]
fn normalized_hash_outputs_stay_in_documented_bounds() {
    for hash in [0, 1, 42, u64::MAX / 2, u64::MAX - 1, u64::MAX] {
        let unit = hash_to_unit_f32(hash);
        let signed = hash_to_signed_f32(hash);

        assert!((0.0..=1.0).contains(&unit));
        assert!((-1.0..=1.0).contains(&signed));
    }
}

#[test]
fn interpolation_helpers_cover_endpoints_and_midpoints() {
    assert_eq!(lerp_f32(2.0, 6.0, 0.0), 2.0);
    assert_eq!(lerp_f32(2.0, 6.0, 1.0), 6.0);
    assert_eq!(lerp_f32(2.0, 6.0, 0.5), 4.0);

    assert_eq!(lerp_f64(-2.0, 2.0, 0.0), -2.0);
    assert_eq!(lerp_f64(-2.0, 2.0, 1.0), 2.0);
    assert_eq!(lerp_f64(-2.0, 2.0, 0.5), 0.0);

    assert_eq!(smoothstep(0.0), 0.0);
    assert_eq!(smoothstep(1.0), 1.0);
    assert_eq!(smootherstep(0.0), 0.0);
    assert_eq!(smootherstep(1.0), 1.0);
}

#[test]
fn interpolation_curves_are_monotonic_for_representative_samples() {
    let mut previous_smooth = smoothstep(0.0);
    let mut previous_smoother = smootherstep(0.0);

    for step in 1..=20 {
        let t = f64::from(step) / 20.0;
        let smooth = smoothstep(t);
        let smoother = smootherstep(t);

        assert!(smooth >= previous_smooth);
        assert!(smoother >= previous_smoother);

        previous_smooth = smooth;
        previous_smoother = smoother;
    }
}

#[test]
fn noise_traits_are_object_safe() {
    let noise_2d = ValueNoise2D::new(GenerationSeed::new(1));
    let noise_3d = ValueNoise3D::new(GenerationSeed::new(1));

    let dyn_2d: &dyn Noise2D = &noise_2d;
    let dyn_3d: &dyn Noise3D = &noise_3d;

    assert!(dyn_2d.sample(1.0, 2.0).is_finite());
    assert!(dyn_3d.sample(1.0, 2.0, 3.0).is_finite());
}

#[test]
fn fractal_default_settings_validate() {
    assert_eq!(
        FractalSettings::default().validate(),
        Ok(FractalSettings::default())
    );
}

#[test]
fn fractal_validation_rejects_invalid_octave_counts() {
    let mut settings = FractalSettings {
        octaves: 0,
        ..FractalSettings::default()
    };

    assert_eq!(settings.validate(), Err(AlgorithmError::InvalidOctaveCount));

    settings = FractalSettings {
        octaves: 33,
        ..FractalSettings::default()
    };

    assert_eq!(settings.validate(), Err(AlgorithmError::InvalidOctaveCount));
}

#[test]
fn fractal_validation_rejects_non_finite_settings() {
    let mut settings = FractalSettings {
        frequency: f64::NAN,
        ..FractalSettings::default()
    };
    assert_eq!(
        settings.validate(),
        Err(AlgorithmError::NonFiniteSetting { field: "frequency" })
    );

    settings = FractalSettings {
        lacunarity: f64::INFINITY,
        ..FractalSettings::default()
    };
    assert_eq!(
        settings.validate(),
        Err(AlgorithmError::NonFiniteSetting {
            field: "lacunarity"
        })
    );

    settings = FractalSettings {
        persistence: f32::NAN,
        ..FractalSettings::default()
    };
    assert_eq!(
        settings.validate(),
        Err(AlgorithmError::NonFiniteSetting {
            field: "persistence"
        })
    );

    settings = FractalSettings {
        amplitude: f32::INFINITY,
        ..FractalSettings::default()
    };
    assert_eq!(
        settings.validate(),
        Err(AlgorithmError::NonFiniteSetting { field: "amplitude" })
    );
}

#[test]
fn fractal_validation_rejects_invalid_numeric_ranges() {
    let mut settings = FractalSettings {
        frequency: 0.0,
        ..FractalSettings::default()
    };
    assert_eq!(settings.validate(), Err(AlgorithmError::InvalidFrequency));

    settings = FractalSettings {
        lacunarity: 0.0,
        ..FractalSettings::default()
    };
    assert_eq!(settings.validate(), Err(AlgorithmError::InvalidLacunarity));

    settings = FractalSettings {
        persistence: -0.1,
        ..FractalSettings::default()
    };
    assert_eq!(settings.validate(), Err(AlgorithmError::InvalidPersistence));
}

#[test]
fn fractal_zero_amplitude_produces_zero() {
    let noise = ValueNoise2D::new(GenerationSeed::new(1));
    let settings = FractalSettings {
        amplitude: 0.0,
        ..FractalSettings::default()
    };

    assert_eq!(fractal_sample_2d(&noise, settings, 1.0, 2.0), Ok(0.0));
}

#[test]
fn fractal_sampling_is_deterministic_and_coordinate_sensitive() {
    let noise = ValueNoise2D::new(GenerationSeed::new(99));
    let settings = FractalSettings::default();

    let first = fractal_sample_2d(&noise, settings, 10.0, -20.0).unwrap();
    let second = fractal_sample_2d(&noise, settings, 10.0, -20.0).unwrap();
    let shifted = fractal_sample_2d(&noise, settings, 10.5, -20.0).unwrap();

    assert_eq!(first, second);
    assert_ne!(first, shifted);
}

#[test]
fn normalized_fractal_sampling_stays_in_expected_range() {
    let noise = ValueNoise3D::new(GenerationSeed::new(55));
    let settings = FractalSettings::default();

    let value = fractal_sample_3d(&noise, settings, -12.0, 4.0, 8.0).unwrap();

    assert!((-1.0..=1.0).contains(&value));
}

#[test]
fn non_normalized_fractal_amplitude_affects_scale() {
    let noise = ConstantNoise(0.25);
    let low = FractalSettings {
        octaves: 2,
        amplitude: 1.0,
        normalize: false,
        ..FractalSettings::default()
    };
    let high = FractalSettings {
        amplitude: 2.0,
        ..low
    };

    let low_value = fractal_sample_2d(&noise, low, 1.0, 2.0).unwrap();
    let high_value = fractal_sample_2d(&noise, high, 1.0, 2.0).unwrap();

    assert_eq!(high_value, low_value * 2.0);
    assert_eq!(noise.0, 0.25);
}

#[test]
fn fractal_sampling_rejects_non_finite_coordinates() {
    let noise = ValueNoise2D::new(GenerationSeed::new(1));

    assert_eq!(
        fractal_sample_2d(&noise, FractalSettings::default(), f64::NAN, 0.0),
        Err(AlgorithmError::NonFiniteCoordinate)
    );
}

#[derive(Debug)]
struct ConstantNoise(f32);

impl Noise2D for ConstantNoise {
    fn sample(&self, _x: f64, _y: f64) -> f32 {
        self.0
    }
}
