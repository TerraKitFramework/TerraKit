use terrakit_algorithms::{
    AlgorithmError,
    noise::{
        Noise2D, Noise3D, NoiseAlgorithm, PerlinNoise2D, PerlinNoise3D, SimplexNoise2D,
        SimplexNoise3D, WorleyNoise2D, WorleyNoise3D,
    },
};
use terrakit_core::GenerationSeed;

const ALGORITHMS: [NoiseAlgorithm; 4] = [
    NoiseAlgorithm::Value,
    NoiseAlgorithm::Perlin,
    NoiseAlgorithm::Simplex,
    NoiseAlgorithm::Worley,
];

#[test]
fn first_party_noise_algorithms_are_deterministic() {
    let seed = GenerationSeed::new(123);

    for algorithm in ALGORITHMS {
        let first_2d = algorithm.create_2d(seed);
        let second_2d = algorithm.create_2d(seed);
        assert_eq!(
            first_2d.sample(10.25, -3.5),
            second_2d.sample(10.25, -3.5),
            "{algorithm:?} 2D was not deterministic"
        );

        let first_3d = algorithm.create_3d(seed);
        let second_3d = algorithm.create_3d(seed);
        assert_eq!(
            first_3d.sample(10.25, -3.5, 8.75),
            second_3d.sample(10.25, -3.5, 8.75),
            "{algorithm:?} 3D was not deterministic"
        );
    }
}

#[test]
fn first_party_noise_algorithms_are_seed_sensitive() {
    let coordinates_2d = [
        (12.345, -67.89),
        (-0.25, 1.75),
        (100.5, -200.25),
        (3.0, 4.0),
    ];
    let coordinates_3d = [
        (12.345, -67.89, 0.25),
        (-0.25, 1.75, 5.5),
        (100.5, -200.25, 33.0),
        (3.0, 4.0, 5.0),
    ];

    for algorithm in ALGORITHMS {
        let first_2d = algorithm.create_2d(GenerationSeed::new(1));
        let second_2d = algorithm.create_2d(GenerationSeed::new(2));
        assert!(
            coordinates_2d
                .iter()
                .any(|(x, y)| first_2d.sample(*x, *y) != second_2d.sample(*x, *y)),
            "{algorithm:?} 2D did not change for representative seed comparison"
        );

        let first_3d = algorithm.create_3d(GenerationSeed::new(1));
        let second_3d = algorithm.create_3d(GenerationSeed::new(2));
        assert!(
            coordinates_3d
                .iter()
                .any(|(x, y, z)| first_3d.sample(*x, *y, *z) != second_3d.sample(*x, *y, *z)),
            "{algorithm:?} 3D did not change for representative seed comparison"
        );
    }
}

#[test]
fn first_party_noise_algorithms_handle_negative_and_large_finite_coordinates() {
    for algorithm in ALGORITHMS {
        let noise_2d = algorithm.create_2d(GenerationSeed::new(42));
        let noise_3d = algorithm.create_3d(GenerationSeed::new(42));

        assert!(noise_2d.sample(-10.75, -20.25).is_finite());
        assert!(noise_3d.sample(-10.75, -20.25, -30.5).is_finite());
        assert!(noise_2d.sample(1.0e12, -1.0e12).is_finite());
        assert!(noise_3d.sample(1.0e12, -1.0e12, 5.0e11).is_finite());
    }
}

#[test]
fn first_party_noise_algorithms_stay_bounded_for_representative_samples() {
    for algorithm in ALGORITHMS {
        let noise_2d = algorithm.create_2d(GenerationSeed::new(88));
        let noise_3d = algorithm.create_3d(GenerationSeed::new(88));

        for step in -16..=16 {
            let coordinate = f64::from(step) * 0.173;
            let value_2d = noise_2d.sample(coordinate, coordinate - 1.5);
            let value_3d = noise_3d.sample(coordinate, coordinate - 1.5, coordinate + 2.0);

            assert!(value_2d.is_finite(), "{algorithm:?} 2D returned {value_2d}");
            assert!(value_3d.is_finite(), "{algorithm:?} 3D returned {value_3d}");
            assert!(
                (-1.25..=1.25).contains(&value_2d),
                "{algorithm:?} 2D returned out-of-family value {value_2d}"
            );
            assert!(
                (-1.25..=1.25).contains(&value_3d),
                "{algorithm:?} 3D returned out-of-family value {value_3d}"
            );
        }
    }
}

#[test]
fn new_noise_algorithms_return_nan_and_validated_samples_return_error_for_non_finite_inputs() {
    let seed = GenerationSeed::new(1);

    let perlin_2d = PerlinNoise2D::new(seed);
    let perlin_3d = PerlinNoise3D::new(seed);
    assert!(perlin_2d.sample(f64::NAN, 0.0).is_nan());
    assert!(perlin_3d.sample(0.0, f64::INFINITY, 0.0).is_nan());
    assert_eq!(
        perlin_2d.try_sample(f64::NAN, 0.0),
        Err(AlgorithmError::NonFiniteCoordinate)
    );
    assert_eq!(
        perlin_3d.try_sample(0.0, f64::INFINITY, 0.0),
        Err(AlgorithmError::NonFiniteCoordinate)
    );

    let simplex_2d = SimplexNoise2D::new(seed);
    let simplex_3d = SimplexNoise3D::new(seed);
    assert!(simplex_2d.sample(f64::NAN, 0.0).is_nan());
    assert!(simplex_3d.sample(0.0, f64::INFINITY, 0.0).is_nan());
    assert_eq!(
        simplex_2d.try_sample(f64::NAN, 0.0),
        Err(AlgorithmError::NonFiniteCoordinate)
    );
    assert_eq!(
        simplex_3d.try_sample(0.0, f64::INFINITY, 0.0),
        Err(AlgorithmError::NonFiniteCoordinate)
    );

    let worley_2d = WorleyNoise2D::new(seed);
    let worley_3d = WorleyNoise3D::new(seed);
    assert!(worley_2d.sample(f64::NAN, 0.0).is_nan());
    assert!(worley_3d.sample(0.0, f64::INFINITY, 0.0).is_nan());
    assert_eq!(
        worley_2d.try_sample(f64::NAN, 0.0),
        Err(AlgorithmError::NonFiniteCoordinate)
    );
    assert_eq!(
        worley_3d.try_sample(0.0, f64::INFINITY, 0.0),
        Err(AlgorithmError::NonFiniteCoordinate)
    );
}

#[test]
fn worley_noise_uses_signed_distance_mapping() {
    let noise = WorleyNoise2D::new(GenerationSeed::new(123));
    let mut saw_negative = false;
    let mut saw_positive = false;

    for y in -8..=8 {
        for x in -8..=8 {
            let value = noise.sample(f64::from(x) * 0.25, f64::from(y) * 0.25);

            assert!((-1.0..=1.0).contains(&value));
            saw_negative |= value < 0.0;
            saw_positive |= value > 0.0;
        }
    }

    assert!(saw_negative);
    assert!(saw_positive);
}
