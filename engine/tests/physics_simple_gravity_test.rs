//! Simple gravity test without collision

use engine::physics::avbd_solver::{AVBDConfig, AVBDSolver, RigidbodyData};
use glam::{Mat3, Quat, Vec3};
use tracing::info;

#[test]
fn test_simple_gravity() {
    // Initialize logging
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    // Create a world to get a valid entity
    let mut world = engine::core::entity::World::new();
    let entity = world.spawn(());

    // Create a single falling body
    let body = RigidbodyData::new(
        entity,
        Vec3::new(0.0, 10.0, 0.0),                    // position
        Quat::IDENTITY,                               // rotation
        Vec3::ZERO,                                   // linear velocity
        Vec3::ZERO,                                   // angular velocity
        1.0,                                          // mass
        Mat3::from_diagonal(Vec3::splat(0.16666667)), // inertia
        true,                                         // use gravity
        false,                                        // not kinematic
        0.01,                                         // linear damping
        0.01,                                         // angular damping
    );

    // Create solver with default config
    let config = AVBDConfig::default();
    let mut solver = AVBDSolver::new(config);

    info!("Initial position: {:?}", body.position);
    info!("Gravity: {:?}", solver.config.gravity);

    // Run physics for 1 second (60 steps at 60Hz)
    let dt = 1.0 / 60.0;
    let mut bodies = vec![body];

    // Update coloring for the single body
    solver.update_coloring(&bodies);

    for i in 0..60 {
        let _pos_before = bodies[0].position;
        let _vel_before = bodies[0].linear_velocity;

        // Run one physics step
        solver.step(&mut bodies, dt);

        if i % 10 == 0 || i < 5 {
            info!(
                "Step {}: pos={:.4}, {:.4}, {:.4} vel={:.4}, {:.4}, {:.4}",
                i,
                bodies[0].position.x,
                bodies[0].position.y,
                bodies[0].position.z,
                bodies[0].linear_velocity.x,
                bodies[0].linear_velocity.y,
                bodies[0].linear_velocity.z
            );
        }
    }

    // After 1 second of falling:
    // Expected position: y = 10 + 0*1 + 0.5*(-9.81)*1² = 10 - 4.905 = 5.095
    // Expected velocity: v = 0 + (-9.81)*1 = -9.81

    info!("Final position: {:?}", bodies[0].position);
    info!("Final velocity: {:?}", bodies[0].linear_velocity);

    assert!(
        (bodies[0].position.y - 5.095).abs() < 0.5,
        "Body should have fallen to around y=5.095, but is at y={}",
        bodies[0].position.y
    );
    assert!(
        (bodies[0].linear_velocity.y + 9.81).abs() < 1.0,
        "Body should have velocity around -9.81, but has {}",
        bodies[0].linear_velocity.y
    );
}
