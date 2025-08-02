//! Tests for the audio system

#[cfg(test)]
mod tests {
    use crate::audio::*;
    use crate::core::entity::*;
    use crate::graphics::culling::AABB;
    use glam::Vec3;

    #[test]
    fn test_audio_ray_aabb_intersection() {
        let ray = AudioRay {
            origin: Vec3::new(0.0, 0.0, -5.0),
            direction: Vec3::new(0.0, 0.0, 1.0),
        };

        let aabb = AABB {
            min: Vec3::new(-1.0, -1.0, -1.0),
            max: Vec3::new(1.0, 1.0, 1.0),
        };

        let hit = raycast::ray_aabb_intersection(&ray, &aabb, 10.0);
        assert!(hit.is_some());
        assert!((hit.unwrap() - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_audio_components() {
        // Test AudioSource
        let mut source = AudioSource::default();
        assert_eq!(source.volume, 1.0);
        assert_eq!(source.pitch, 1.0);
        assert!(!source.looping);
        assert!(source.spatial);

        // Test AudioListener
        let listener = AudioListener::default();
        assert!(listener.active);
        assert_eq!(listener.master_volume, 1.0);

        // Test AmbientSound
        let ambient = AmbientSound::default();
        assert_eq!(ambient.volume, 0.5);
        assert!(ambient.looping);
        assert!(ambient.auto_play);

        // Test AudioMaterial presets
        let concrete = AudioMaterial::concrete();
        assert!(concrete.absorption < 0.1);
        assert_eq!(concrete.transmission, 0.0);

        let glass = AudioMaterial::glass();
        assert!(glass.transmission > 0.0);
    }

    #[test]
    fn test_occlusion_calculation() {
        let mut world = World::new();

        // Create occluder entity
        let occluder = world.spawn();
        world
            .insert(
                occluder,
                (
                    Transform::from_position(Vec3::new(0.0, 0.0, 0.0)),
                    AABB {
                        min: Vec3::new(-1.0, -1.0, -1.0),
                        max: Vec3::new(1.0, 1.0, 1.0),
                    },
                    AudioMaterial::concrete(),
                ),
            )
            .unwrap();

        // Create source entity
        let source = world.spawn();
        world
            .insert(
                source,
                (
                    Transform::from_position(Vec3::new(5.0, 0.0, 0.0)),
                    AABB {
                        min: Vec3::new(-0.5, -0.5, -0.5),
                        max: Vec3::new(0.5, 0.5, 0.5),
                    },
                ),
            )
            .unwrap();

        // Test occlusion from different listener positions
        let listener_pos1 = Vec3::new(-5.0, 0.0, 0.0);
        let source_pos = Vec3::new(5.0, 0.0, 0.0);

        let occlusion = propagation::calculate_occlusion(listener_pos1, source_pos, &world, source);

        // Should be occluded by the concrete block
        assert!(occlusion > 0.9);

        // Test from a position with clear line of sight
        let listener_pos2 = Vec3::new(3.0, 0.0, 0.0);
        let occlusion2 =
            propagation::calculate_occlusion(listener_pos2, source_pos, &world, source);

        // Should have no occlusion
        assert_eq!(occlusion2, 0.0);
    }

    #[test]
    fn test_distance_attenuation() {
        use crate::audio::source::calculate_distance_attenuation;

        // Test at reference distance
        let att1 = calculate_distance_attenuation(1.0, 50.0, 1.0);
        assert!((att1 - 1.0).abs() < 0.001);

        // Test at half max distance
        let att2 = calculate_distance_attenuation(25.0, 50.0, 1.0);
        assert!(att2 < 1.0);
        assert!(att2 > 0.0);

        // Test beyond max distance
        let att3 = calculate_distance_attenuation(60.0, 50.0, 1.0);
        assert_eq!(att3, 0.0);

        // Test with different rolloff factors
        let att4 = calculate_distance_attenuation(10.0, 50.0, 2.0);
        let att5 = calculate_distance_attenuation(10.0, 50.0, 0.5);
        assert!(att4 < att5); // Higher rolloff = faster attenuation
    }

    #[test]
    fn test_room_acoustics() {
        let mut world = World::new();

        // Create a simple room with walls
        let wall_positions = vec![
            Vec3::new(10.0, 0.0, 0.0),  // Right wall
            Vec3::new(-10.0, 0.0, 0.0), // Left wall
            Vec3::new(0.0, 0.0, 10.0),  // Front wall
            Vec3::new(0.0, 0.0, -10.0), // Back wall
            Vec3::new(0.0, 5.0, 0.0),   // Ceiling
            Vec3::new(0.0, -5.0, 0.0),  // Floor
        ];

        for (i, pos) in wall_positions.iter().enumerate() {
            let wall = world.spawn();
            world
                .insert(
                    wall,
                    (
                        Transform::from_position(*pos),
                        AABB {
                            min: Vec3::new(-15.0, -15.0, -15.0),
                            max: Vec3::new(15.0, 15.0, 15.0),
                        },
                        if i < 4 {
                            AudioMaterial::concrete()
                        } else {
                            AudioMaterial::wood()
                        },
                    ),
                )
                .unwrap();
        }

        // Test room acoustics from center
        let acoustics = propagation::estimate_room_acoustics(Vec3::ZERO, &world);

        // Should detect room size approximately
        assert!(acoustics.size > 5.0);
        assert!(acoustics.size < 15.0);

        // Should calculate reverb time
        assert!(acoustics.reverb_time > 0.1);
        assert!(acoustics.reverb_time < 10.0);

        // Should have some early reflections
        assert!(!acoustics.early_reflections.is_empty());
    }

    #[test]
    fn test_listener_state() {
        use crate::audio::listener::ListenerState;

        let mut transform = Transform::default();
        transform.position = Vec3::new(10.0, 5.0, 0.0);
        transform.rotation = glam::Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);

        let state = ListenerState::from_transform(
            Entity::from_bits(1).unwrap(),
            &transform,
            Vec3::new(1.0, 0.0, 0.0),
            0.8,
        );

        assert_eq!(state.position, Vec3::new(10.0, 5.0, 0.0));
        assert_eq!(state.master_volume, 0.8);

        // Check that rotation affects orientation vectors
        assert!((state.forward - Vec3::X).length() < 0.001);
        assert!((state.right - Vec3::Z).length() < 0.001);
    }
}
