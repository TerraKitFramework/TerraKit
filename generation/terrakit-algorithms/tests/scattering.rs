use terrakit_algorithms::scattering::{PoissonSettings, ScatterArea, scatter_poisson};

#[test]
fn test_poisson_scattering() {
    let area = ScatterArea::new(0.0, 100.0, 0.0, 100.0);

    let settings = PoissonSettings {
        seed: 12345,
        radius: 5.0,
        attempts: 30,
        prototype_id: 0,
        min_scale: 0.8,
        max_scale: 1.3,
        random_rotation: true,
    };

    let points = scatter_poisson(settings, area).unwrap();

    assert!(!points.is_empty());

    println!("Generated {} points", points.len());
}

#[test]
fn poisson_is_deterministic_and_respects_minimum_spacing() {
    let settings = PoissonSettings {
        seed: 173,
        radius: 2.25,
        ..PoissonSettings::default()
    };
    let area = ScatterArea::new(0.0, 20.0, 0.0, 20.0);
    let first = scatter_poisson(settings, area).unwrap();
    let second = scatter_poisson(settings, area).unwrap();
    assert_eq!(first, second);
    for (i, a) in first.iter().enumerate() {
        for b in &first[i + 1..] {
            let dx = a.position[0] - b.position[0];
            let dz = a.position[2] - b.position[2];
            assert!(dx * dx + dz * dz >= (settings.radius - 1e-4).powi(2));
        }
    }
}

#[test]
fn poisson_rejects_grid_allocation_explosion() {
    let settings = PoissonSettings {
        radius: 0.01,
        ..PoissonSettings::default()
    };
    let result = scatter_poisson(settings, ScatterArea::new(0.0, 1000.0, 0.0, 1000.0));
    assert_eq!(
        result,
        Err(terrakit_algorithms::scattering::ScatterError::GridTooLarge)
    );
}
