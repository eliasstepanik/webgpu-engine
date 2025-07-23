//! Standalone debug test for collision detection issue

use engine::core::entity::{components::GlobalTransform, Transform, World};
use engine::physics::{
    collision::{broad_phase::BroadPhaseEntry, narrow_phase::test_collision, AABB},
    components::{Collider, CollisionShape},
};
use glam::{Quat, Vec3};

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("engine::physics=trace")
        .init();

    println!("=== Testing Collision Detection ===\n");

    // Test 1: Direct collision test without scaling
    println!("Test 1: Direct collision test (no scaling)");
    {
        let mut dummy_world = hecs::World::new();
        let floor_entity = dummy_world.spawn(());
        let box_entity = dummy_world.spawn(());

        let floor_shape = CollisionShape::Box {
            half_extents: Vec3::new(10.0, 0.1, 10.0),
        };
        let box_shape = CollisionShape::Box {
            half_extents: Vec3::new(0.5, 0.5, 0.5),
        };

        let floor_pos = Vec3::new(0.0, -1.0, 0.0);
        let box_pos = Vec3::new(0.0, -0.4, 0.0); // Should be colliding

        let contact = test_collision(
            &floor_shape,
            (floor_pos, Quat::IDENTITY),
            floor_entity,
            &box_shape,
            (box_pos, Quat::IDENTITY),
            box_entity,
        );

        println!("Floor at y={}, Box at y={}", floor_pos.y, box_pos.y);
        println!("Contact: {:?}\n", contact);
    }

    // Test 2: With the actual scene setup
    println!("Test 2: Scene setup with scaling");
    {
        let mut world = World::new();

        // Create floor
        let floor = world.spawn((
            Transform {
                position: Vec3::new(0.0, -1.0, 0.0),
                scale: Vec3::new(20.0, 0.2, 20.0),
                ..Default::default()
            },
            GlobalTransform::default(),
            Collider {
                shape: CollisionShape::Box {
                    half_extents: Vec3::new(0.5, 0.5, 0.5),
                },
                is_trigger: false,
                material_id: None,
            },
        ));

        // Create box
        let falling_box = world.spawn((
            Transform {
                position: Vec3::new(0.0, 0.0, 0.0), // At origin
                ..Default::default()
            },
            GlobalTransform::default(),
            Collider {
                shape: CollisionShape::Box {
                    half_extents: Vec3::new(0.5, 0.5, 0.5),
                },
                is_trigger: false,
                material_id: None,
            },
        ));

        // Update hierarchy
        engine::core::entity::hierarchy::advance_frame();
        engine::core::entity::update_hierarchy_system(&mut world);

        // Get transforms and colliders
        let floor_global = world.get::<GlobalTransform>(floor).unwrap();
        let floor_collider = world.get::<Collider>(floor).unwrap();
        let (floor_scale, floor_rot, floor_pos) = floor_global.matrix.to_scale_rotation_translation();

        let box_global = world.get::<GlobalTransform>(falling_box).unwrap();
        let box_collider = world.get::<Collider>(falling_box).unwrap();
        let (box_scale, box_rot, box_pos) = box_global.matrix.to_scale_rotation_translation();

        println!("Floor: pos={:?}, scale={:?}", floor_pos, floor_scale);
        println!("Box: pos={:?}, scale={:?}", box_pos, box_scale);

        // Apply scaling to shapes
        let floor_scaled = match &floor_collider.shape {
            CollisionShape::Box { half_extents } => CollisionShape::Box {
                half_extents: *half_extents * floor_scale,
            },
            _ => panic!("Expected box"),
        };

        let box_scaled = match &box_collider.shape {
            CollisionShape::Box { half_extents } => CollisionShape::Box {
                half_extents: *half_extents * box_scale,
            },
            _ => panic!("Expected box"),
        };

        println!("Floor scaled half_extents: {:?}", match &floor_scaled {
            CollisionShape::Box { half_extents } => half_extents,
            _ => panic!(),
        });
        println!("Box scaled half_extents: {:?}", match &box_scaled {
            CollisionShape::Box { half_extents } => half_extents,
            _ => panic!(),
        });

        // Test collision
        let contact = test_collision(
            &floor_scaled,
            (floor_pos, floor_rot),
            floor,
            &box_scaled,
            (box_pos, box_rot),
            falling_box,
        );

        println!("Contact: {:?}\n", contact);

        // Test AABBs
        let floor_aabb = floor_scaled.world_aabb(floor_pos, floor_rot);
        let box_aabb = box_scaled.world_aabb(box_pos, box_rot);

        println!("Floor AABB: min={:?}, max={:?}", floor_aabb.min, floor_aabb.max);
        println!("Box AABB: min={:?}, max={:?}", box_aabb.min, box_aabb.max);
        println!("AABBs overlap: {}", floor_aabb.overlaps(&box_aabb));
    }

    // Test 3: Sweep and prune
    println!("\nTest 3: Broad phase detection");
    {
        let floor_aabb = AABB::new(
            Vec3::new(-10.0, -1.1, -10.0),
            Vec3::new(10.0, -0.9, 10.0),
        );
        let box_aabb = AABB::new(
            Vec3::new(-0.5, -0.5, -0.5),
            Vec3::new(0.5, 0.5, 0.5),
        );

        println!("Floor AABB: min={:?}, max={:?}", floor_aabb.min, floor_aabb.max);
        println!("Box AABB: min={:?}, max={:?}", box_aabb.min, box_aabb.max);
        println!("Overlap: {}", floor_aabb.overlaps(&box_aabb));

        let mut dummy_world = hecs::World::new();
        let floor_entity = dummy_world.spawn(());
        let box_entity = dummy_world.spawn(());

        let entries = vec![
            BroadPhaseEntry {
                entity: floor_entity,
                aabb: floor_aabb,
            },
            BroadPhaseEntry {
                entity: box_entity,
                aabb: box_aabb,
            },
        ];

        let pairs = engine::physics::collision::broad_phase::sweep_and_prune(&entries);
        println!("Broad phase pairs: {:?}", pairs);
    }
}