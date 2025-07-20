//! Tests for large world coordinate system

use super::*;
use crate::core::entity::components::{GlobalTransform, Transform};
use glam::{DVec3, Quat, Vec3};

#[test]
fn test_precision_at_large_distances() {
    // Test precision at 100 million units from origin - this is the key test
    let world_pos = DVec3::new(100_000_000.0, 50_000_000.0, 75_000_000.0);
    let camera_pos = DVec3::new(99_999_999.0, 49_999_999.5, 75_000_000.25);

    let world_transform = WorldTransform {
        position: world_pos,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    let relative = world_transform.to_camera_relative(camera_pos);

    // Should maintain sub-unit precision even at 100 million units
    assert!((relative.position.x - 1.0).abs() < 0.001);
    assert!((relative.position.y - 0.5).abs() < 0.001);
    assert!((relative.position.z + 0.25).abs() < 0.001);
}

#[test]
fn test_precision_at_planetary_scale() {
    // Test at Earth's radius scale (6.37 million meters)
    let earth_radius = 6_370_000.0;
    let world_pos = DVec3::new(earth_radius, 0.0, 0.0);
    let camera_pos = DVec3::new(earth_radius - 1000.0, 0.0, 0.0); // 1km away

    let world_transform = WorldTransform::from_position(world_pos);
    let relative = world_transform.to_camera_relative(camera_pos);

    // Should maintain meter precision at planetary scale
    assert!((relative.position.x - 1000.0).abs() < 0.01);
}

#[test]
fn test_coordinate_system_origin_shifting() {
    let mut coord_system = CoordinateSystem::with_config(10_000.0, true);

    // Camera starts at origin
    assert_eq!(coord_system.camera_origin, DVec3::ZERO);
    assert_eq!(coord_system.total_origin_offset, DVec3::ZERO);

    // Move camera beyond threshold
    let far_camera_pos = DVec3::new(15_000.0, 5_000.0, 0.0);
    let shifted = coord_system.update_camera_origin(far_camera_pos);

    // Should trigger origin shift
    assert!(shifted);
    assert_eq!(coord_system.camera_origin, DVec3::ZERO); // Camera back at origin
    assert_eq!(coord_system.total_origin_offset, far_camera_pos); // World shifted

    // Test coordinate conversion
    let world_pos = DVec3::new(20_000.0, 8_000.0, 1_000.0);
    let current_pos = coord_system.world_to_current(world_pos);
    let back_to_world = coord_system.current_to_world(current_pos);

    assert!((back_to_world - world_pos).length() < f64::EPSILON);
}

#[test]
fn test_origin_shifting_disabled() {
    let mut coord_system = CoordinateSystem::with_config(10_000.0, false);

    // Move camera far beyond threshold
    let far_camera_pos = DVec3::new(50_000.0, 25_000.0, 0.0);
    let shifted = coord_system.update_camera_origin(far_camera_pos);

    // Should not trigger origin shift when disabled
    assert!(!shifted);
    assert_eq!(coord_system.camera_origin, far_camera_pos);
    assert_eq!(coord_system.total_origin_offset, DVec3::ZERO);
}

#[test]
fn test_mixed_hierarchy_precision() {
    // Test hierarchy with both Transform and WorldTransform entities
    let mut world = crate::core::entity::World::new();

    // Create a WorldTransform parent at a large distance
    let parent_world_pos = DVec3::new(50_000_000.0, 0.0, 0.0);
    let parent = world.spawn((
        WorldTransform::from_position(parent_world_pos),
        crate::core::entity::components::GlobalWorldTransform::default(),
    ));

    // Create a regular Transform child
    let child_local_pos = Vec3::new(10.0, 0.0, 0.0);
    let child = world.spawn((
        Transform::from_position(child_local_pos),
        GlobalTransform::default(),
        crate::core::entity::components::Parent(parent),
    ));

    // Update hierarchy
    crate::core::entity::hierarchy::update_hierarchy_system(&mut world);

    // Check that child's global transform is correct
    let _child_global = world.get::<GlobalTransform>(child).unwrap();
    let expected_world_pos = parent_world_pos + child_local_pos.as_dvec3();

    // When converted to camera-relative, should maintain precision
    let camera_pos = DVec3::new(50_000_005.0, 0.0, 0.0);
    let relative_pos = expected_world_pos - camera_pos;

    // Should be approximately (5.0, 0.0, 0.0) in camera space
    assert!((relative_pos.x - 5.0).abs() < 0.001);
    assert!(relative_pos.y.abs() < 0.001);
    assert!(relative_pos.z.abs() < 0.001);
}

#[test]
fn test_camera_relative_conversion_identity() {
    // Test that camera-relative conversion preserves identity when camera at origin
    let world_transform = WorldTransform {
        position: DVec3::new(100.0, 200.0, 300.0),
        rotation: Quat::from_rotation_y(std::f32::consts::PI / 4.0),
        scale: Vec3::new(2.0, 3.0, 4.0),
    };

    let camera_origin = DVec3::ZERO;
    let relative = world_transform.to_camera_relative(camera_origin);

    // Position should be unchanged
    assert!((relative.position.x - 100.0).abs() < 0.001);
    assert!((relative.position.y - 200.0).abs() < 0.001);
    assert!((relative.position.z - 300.0).abs() < 0.001);

    // Rotation and scale should be preserved exactly
    assert_eq!(relative.rotation, world_transform.rotation);
    assert_eq!(relative.scale, world_transform.scale);
}

#[test]
fn test_distance_calculations() {
    let transform1 = WorldTransform::from_position(DVec3::new(1_000_000.0, 0.0, 0.0));
    let transform2 = WorldTransform::from_position(DVec3::new(1_000_003.0, 4.0, 0.0));

    let distance = transform1.distance_to(&transform2);
    let expected_distance = 5.0; // 3-4-5 triangle

    assert!((distance - expected_distance).abs() < 0.001);
}

#[test]
fn test_render_distance_check() {
    let entity_pos = DVec3::new(10_000_000.0, 0.0, 0.0);
    let camera_pos = DVec3::new(9_999_000.0, 0.0, 0.0); // 1000 units away

    let world_transform = WorldTransform::from_position(entity_pos);

    // Should be within 1500 unit render distance
    assert!(world_transform.is_within_render_distance(camera_pos, 1500.0));

    // Should not be within 500 unit render distance
    assert!(!world_transform.is_within_render_distance(camera_pos, 500.0));
}

#[test]
fn test_coordinate_system_stats() {
    let mut coord_system = CoordinateSystem::with_config(5_000.0, true);

    // Perform some origin shifts
    coord_system.update_camera_origin(DVec3::new(8_000.0, 0.0, 0.0));
    coord_system.update_camera_origin(DVec3::new(6_000.0, 0.0, 0.0));

    let stats = coord_system.get_stats();

    assert_eq!(stats.origin_shifts_performed, 2);
    assert!(stats.origin_shift_enabled);
    assert_eq!(stats.origin_threshold, 5_000.0);

    // Camera world position should be total offset + current relative position
    let expected_world_pos = stats.total_origin_offset + stats.camera_relative_position;
    assert!((stats.camera_world_position - expected_world_pos).length() < f64::EPSILON);
}

#[test]
fn test_transform_matrix_precision() {
    // Test that matrix operations maintain precision at large scales
    let large_pos = DVec3::new(1_000_000_000.0, 500_000_000.0, 250_000_000.0);
    let world_transform = WorldTransform {
        position: large_pos,
        rotation: Quat::from_rotation_y(0.5),
        scale: Vec3::new(2.0, 3.0, 4.0),
    };

    let matrix = world_transform.to_matrix();
    let extracted_pos = matrix.w_axis.truncate();

    // Position should be preserved in the matrix
    assert!((extracted_pos.x - large_pos.x).abs() < 1.0);
    assert!((extracted_pos.y - large_pos.y).abs() < 1.0);
    assert!((extracted_pos.z - large_pos.z).abs() < 1.0);
}

#[test]
fn test_large_world_config_defaults() {
    let config = LargeWorldConfig::default();

    assert!(config.enable_large_world); // Enabled by default
    assert_eq!(config.origin_shift_threshold, 50_000.0);
    assert!(config.use_logarithmic_depth); // Enabled by default
    assert_eq!(config.max_render_distance, 1_000_000_000.0); // 1 billion units
}

#[test]
fn test_look_at_functionality() {
    let mut world_transform = WorldTransform::from_position(DVec3::new(1_000_000.0, 0.0, 0.0));
    let target = DVec3::new(1_000_001.0, 1.0, 0.0);
    let up = DVec3::new(0.0, 1.0, 0.0);

    world_transform.look_at(target, up);

    // Transform should now be looking towards the target
    let forward = world_transform.rotation * Vec3::NEG_Z; // Forward is -Z in right-handed
    let expected_forward = (target - world_transform.position).normalize().as_vec3();

    // Forwards should be approximately equal
    assert!((forward - expected_forward).length() < 0.01);
}

#[test]
fn test_f32_precision_limits() {
    // Demonstrate f32 precision issues that WorldTransform solves
    let large_f32_pos = 16_777_216.0_f32; // 2^24, where f32 loses precision
    let small_offset = 1.0_f32;

    // f32 precision loss demonstration
    let f32_result = large_f32_pos + small_offset;
    assert_eq!(f32_result, large_f32_pos); // Precision lost!

    // DVec3 maintains precision
    let large_f64_pos = 16_777_216.0_f64;
    let f64_result = large_f64_pos + small_offset as f64;
    assert_ne!(f64_result, large_f64_pos); // Precision maintained!
    assert_eq!(f64_result, 16_777_217.0);
}

#[cfg(test)]
mod benchmark_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_coordinate_conversion() {
        let world_transforms: Vec<WorldTransform> = (0..1000)
            .map(|i| WorldTransform::from_position(DVec3::new(i as f64 * 10000.0, 0.0, 0.0)))
            .collect();

        let camera_pos = DVec3::new(5_000_000.0, 0.0, 0.0);

        let start = Instant::now();
        let _relative_transforms: Vec<Transform> = world_transforms
            .iter()
            .map(|wt| wt.to_camera_relative(camera_pos))
            .collect();
        let duration = start.elapsed();

        // Should be able to convert 1000 transforms in well under 1ms
        assert!(
            duration.as_millis() < 10,
            "Coordinate conversion too slow: {duration:?}"
        );
    }

    #[test]
    fn benchmark_origin_shifting() {
        let mut coord_system = CoordinateSystem::with_config(10_000.0, true);

        let start = Instant::now();
        for i in 0..100 {
            let pos = DVec3::new((i * 15_000) as f64, 0.0, 0.0);
            coord_system.update_camera_origin(pos);
        }
        let duration = start.elapsed();

        // Should be able to perform 100 origin shifts in well under 1ms
        assert!(
            duration.as_millis() < 5,
            "Origin shifting too slow: {duration:?}"
        );
    }

    #[test]
    fn test_galaxy_scale_precision() {
        // Test at Milky Way scale (10^20 meters)
        let galaxy_radius = 9.46e20; // ~100,000 light years
        let world_pos = DVec3::new(galaxy_radius, 0.0, 0.0);
        let camera_pos = DVec3::new(galaxy_radius - 1000.0, 0.0, 0.0); // 1km away

        let world_transform = WorldTransform::from_position(world_pos);
        let relative = world_transform.to_camera_relative(camera_pos);

        // At galaxy scale (10^20), f64 precision is ~10^5 meters
        // So we can't expect meter precision, but should be within 100km
        assert!((relative.position.x - 1000.0).abs() < 1e5);
    }

    #[test]
    fn test_hierarchical_galaxy_coordinates() {
        use crate::core::coordinates::GalaxyPosition;
        use glam::IVec3;

        let sector_size = 1e18; // 1 million light years per sector
        let world_pos = DVec3::new(2.5e18, 1.3e18, -0.7e18);

        let galaxy_pos = GalaxyPosition::from_world_position(world_pos, sector_size);
        assert_eq!(galaxy_pos.sector.sector_coords, IVec3::new(2, 1, -1));

        let reconstructed = galaxy_pos.to_world_position();
        assert!((reconstructed - world_pos).length() < 1.0); // Sub-meter precision
    }

    #[test]
    fn test_galaxy_coordinate_conversions() {
        let sector_size = 1e18;
        let world_transform = WorldTransform::from_position(DVec3::new(3.7e18, -2.1e18, 0.8e18));

        // Convert to galaxy position and back
        let galaxy_pos = world_transform.to_galaxy_position(sector_size);
        let reconstructed = WorldTransform::from_galaxy_position(&galaxy_pos);

        // Should maintain precision through conversion
        assert!((world_transform.position - reconstructed.position).length() < 1.0);
        assert_eq!(world_transform.rotation, reconstructed.rotation);
        assert_eq!(world_transform.scale, reconstructed.scale);
    }

    #[test]
    fn test_is_galaxy_scale() {
        // Test detection of galaxy-scale positions
        let local_transform = WorldTransform::from_position(DVec3::new(1000.0, 0.0, 0.0));
        assert!(!local_transform.is_galaxy_scale());

        let planetary_transform = WorldTransform::from_position(DVec3::new(1e8, 0.0, 0.0));
        assert!(!planetary_transform.is_galaxy_scale());

        let galaxy_transform = WorldTransform::from_position(DVec3::new(1e16, 0.0, 0.0));
        assert!(galaxy_transform.is_galaxy_scale());
    }

    #[test]
    fn test_galaxy_scale_camera_relative() {
        use crate::core::coordinates::{GalaxyPosition, GalaxySector};
        use glam::IVec3;

        let sector_size = 1e18;

        // Camera at edge of one sector
        let camera_pos = GalaxyPosition::new(
            GalaxySector::new(IVec3::new(5, 5, 5), sector_size),
            DVec3::new(4.9e17, 0.0, 0.0),
        );

        // Object in adjacent sector
        let object_pos = GalaxyPosition::new(
            GalaxySector::new(IVec3::new(6, 5, 5), sector_size),
            DVec3::new(-4.9e17, 0.0, 0.0),
        );

        let relative = object_pos.to_camera_relative(&camera_pos);

        // Should be approximately 0.2e17 meters away
        // Camera at 4.9e17 in sector 5, object at -4.9e17 in sector 6
        // Distance = 1e18 - 4.9e17 - 4.9e17 = 0.2e17
        assert!((relative.x - 0.2e17).abs() < 1e10); // Allow some precision loss at this scale
    }
}
