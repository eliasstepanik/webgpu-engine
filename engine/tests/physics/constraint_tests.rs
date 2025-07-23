//! Tests for constraint system

use engine::physics::{
    avbd_solver::{AVBDConfig, AVBDSolver, RigidbodyData},
    collision::Contact,
    constraints::{BallJoint, ContactConstraint, Constraint, ConstraintData},
    components::PhysicsMaterial,
};
use glam::{Mat3, Quat, Vec3};
use hecs::Entity;

#[test]
fn test_constraint_data_warmstart() {
    let mut data = ConstraintData::hard(
        Vec3::splat(0.0),
        Vec3::splat(f32::INFINITY),
        100.0,
    );
    
    data.lambda = Vec3::ONE;
    data.k = Vec3::splat(200.0);
    
    data.warm_start(0.9);
    
    assert!((data.lambda - Vec3::splat(0.9)).length() < 0.01);
    assert!((data.k - Vec3::splat(180.0)).length() < 0.01);
}

#[test]
fn test_contact_constraint_creation() {
    let contact = Contact::new(
        Entity::from_bits(0),
        Entity::from_bits(1),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::Y,
        0.1,
    );
    
    let bodies = vec![
        create_test_body(0, Vec3::ZERO),
        create_test_body(1, Vec3::new(0.0, 2.0, 0.0)),
    ];
    
    let material = PhysicsMaterial {
        static_friction: 0.5,
        dynamic_friction: 0.3,
        restitution: 0.2,
    };
    
    let constraint = ContactConstraint::new(contact, 0, 1, &bodies, Some(&material));
    
    assert_eq!(constraint.body_a, 0);
    assert_eq!(constraint.body_b, 1);
    assert_eq!(constraint.friction, 0.3);
    assert_eq!(constraint.restitution, 0.2);
}

#[test]
fn test_contact_constraint_evaluate() {
    let contact = Contact::new(
        Entity::from_bits(0),
        Entity::from_bits(1),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::Y,
        0.1,
    );
    
    let mut bodies = vec![
        create_test_body(0, Vec3::new(0.0, 0.0, 0.0)),
        create_test_body(1, Vec3::new(0.0, 2.0, 0.0)),
    ];
    
    let constraint = ContactConstraint::new(contact, 0, 1, &bodies, None);
    
    // Move bodies apart
    bodies[0].position = Vec3::new(0.0, -1.0, 0.0);
    bodies[1].position = Vec3::new(0.0, 3.0, 0.0);
    
    let c = constraint.evaluate(&bodies);
    // Should have separation in normal direction
    assert!(c.z < 0.0);
}

#[test]
fn test_ball_joint_constraint() {
    let bodies = vec![
        create_test_body(0, Vec3::new(-1.0, 0.0, 0.0)),
        create_test_body(1, Vec3::new(1.0, 0.0, 0.0)),
    ];
    
    let joint = BallJoint::new(0, 1, Vec3::ZERO, &bodies);
    
    // Check that local anchors are correct
    assert_eq!(joint.local_anchor_a, Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(joint.local_anchor_b, Vec3::new(-1.0, 0.0, 0.0));
    
    // Evaluate constraint - should be zero when anchors coincide
    let c = joint.evaluate(&bodies);
    assert!(c.length() < 0.01);
}

#[test]
fn test_ball_joint_separation() {
    let mut bodies = vec![
        create_test_body(0, Vec3::new(-1.0, 0.0, 0.0)),
        create_test_body(1, Vec3::new(1.0, 0.0, 0.0)),
    ];
    
    let joint = BallJoint::new(0, 1, Vec3::ZERO, &bodies);
    
    // Move bodies apart
    bodies[1].position = Vec3::new(2.0, 0.0, 0.0);
    
    // Constraint should now be non-zero
    let c = joint.evaluate(&bodies);
    assert!((c - Vec3::new(1.0, 0.0, 0.0)).length() < 0.01);
}

#[test]
fn test_avbd_solver_creation() {
    let config = AVBDConfig {
        iterations: 10,
        beta: 5.0,
        alpha: 0.9,
        gamma: 0.95,
        k_start: 50.0,
        gravity: Vec3::new(0.0, -10.0, 0.0),
    };
    
    let solver = AVBDSolver::new(config.clone());
    assert_eq!(solver.config.iterations, 10);
    assert_eq!(solver.config.beta, 5.0);
}

#[test]
fn test_avbd_solver_step() {
    let config = AVBDConfig::default();
    let mut solver = AVBDSolver::new(config);
    
    let mut bodies = vec![
        create_test_body(0, Vec3::new(0.0, 10.0, 0.0)),
    ];
    bodies[0].use_gravity = true;
    
    // Step the solver
    solver.step(&mut bodies, 0.016);
    
    // Body should have moved down due to gravity
    assert!(bodies[0].position.y < 10.0);
    assert!(bodies[0].linear_velocity.y < 0.0);
}

#[test]
fn test_constraint_affects_body() {
    let bodies = vec![
        create_test_body(0, Vec3::ZERO),
        create_test_body(1, Vec3::new(2.0, 0.0, 0.0)),
    ];
    
    let joint = BallJoint::new(0, 1, Vec3::new(1.0, 0.0, 0.0), &bodies);
    
    assert!(joint.affects_body(0));
    assert!(joint.affects_body(1));
    assert!(!joint.affects_body(2));
}

#[test]
fn test_constraint_force_and_hessian() {
    let bodies = vec![
        create_test_body(0, Vec3::ZERO),
        create_test_body(1, Vec3::new(2.0, 0.0, 0.0)),
    ];
    
    let mut joint = BallJoint::new(0, 1, Vec3::new(1.0, 0.0, 0.0), &bodies);
    joint.data.k = Vec3::splat(1000.0);
    
    let (force, hessian) = joint.compute_force_and_hessian(0, &bodies);
    
    // Should have non-zero force
    assert!(force.0.length() > 0.0 || force.1.length() > 0.0);
    
    // Hessian should be symmetric positive semi-definite
    // (simplified check - just ensure it's not zero)
    assert!(hessian.a != Mat3::ZERO || hessian.d != Mat3::ZERO);
}

// Helper function to create test rigidbody
fn create_test_body(idx: usize, position: Vec3) -> RigidbodyData {
    RigidbodyData::new(
        Entity::from_bits(idx as u64),
        position,
        Quat::IDENTITY,
        Vec3::ZERO,
        Vec3::ZERO,
        1.0,
        Mat3::IDENTITY,
        false,
        false,
    )
}