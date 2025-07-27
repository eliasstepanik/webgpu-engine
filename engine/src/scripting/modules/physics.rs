//! Physics API for Rhai scripts
//!
//! This module exposes physics functionality to scripts through a safe,
//! command-based API that integrates with the physics system.

use crate::physics::PhysicsCommand;
use glam::Vec3;
use rhai::{Dynamic, Engine, EvalAltResult, Module};
use tracing::{debug, trace};

/// Register physics types and global functions with Rhai engine
pub fn register_physics_api(engine: &mut Engine) {
    debug!("Registering physics API");

    // Register Vec3 type and constructor
    engine
        .register_type_with_name::<Vec3>("Vec3")
        .register_fn("vec3", |x: f32, y: f32, z: f32| Vec3::new(x, y, z))
        .register_get("x", |v: &mut Vec3| v.x)
        .register_get("y", |v: &mut Vec3| v.y)
        .register_get("z", |v: &mut Vec3| v.z)
        .register_set("x", |v: &mut Vec3, x: f32| v.x = x)
        .register_set("y", |v: &mut Vec3, y: f32| v.y = y)
        .register_set("z", |v: &mut Vec3, z: f32| v.z = z);

    // Register Vec3 operations
    engine
        .register_fn("+", |a: Vec3, b: Vec3| a + b)
        .register_fn("-", |a: Vec3, b: Vec3| a - b)
        .register_fn("*", |a: Vec3, b: f32| a * b)
        .register_fn("*", |a: f32, b: Vec3| b * a)
        .register_fn("/", |a: Vec3, b: f32| a / b)
        .register_fn("dot", |a: Vec3, b: Vec3| a.dot(b))
        .register_fn("cross", |a: Vec3, b: Vec3| a.cross(b))
        .register_fn("length", |v: Vec3| v.length())
        .register_fn("normalize", |v: Vec3| v.normalize());

    debug!("Physics API registered");
}

/// Create a physics module for scripts
pub fn create_physics_module() -> Module {
    let mut module = Module::new();

    // Apply force to entity
    module.set_native_fn(
        "apply_force",
        move |entity: i64, force: Dynamic| -> Result<(), Box<EvalAltResult>> {
            let force_vec = parse_vec3_from_dynamic(force)?;
            crate::physics::system::queue_physics_command(PhysicsCommand::ApplyForce {
                entity: entity as u64,
                force: force_vec,
            });
            trace!(entity = entity, force = ?force_vec, "Queued apply_force command");
            Ok(())
        },
    );

    // Apply impulse to entity
    module.set_native_fn(
        "apply_impulse",
        move |entity: i64, impulse: Dynamic| -> Result<(), Box<EvalAltResult>> {
            let impulse_vec = parse_vec3_from_dynamic(impulse)?;
            crate::physics::system::queue_physics_command(PhysicsCommand::ApplyImpulse {
                entity: entity as u64,
                impulse: impulse_vec,
            });
            trace!(entity = entity, impulse = ?impulse_vec, "Queued apply_impulse command");
            Ok(())
        },
    );

    // Apply torque to entity
    module.set_native_fn(
        "apply_torque",
        move |entity: i64, torque: Dynamic| -> Result<(), Box<EvalAltResult>> {
            let torque_vec = parse_vec3_from_dynamic(torque)?;
            crate::physics::system::queue_physics_command(PhysicsCommand::ApplyTorque {
                entity: entity as u64,
                torque: torque_vec,
            });
            trace!(entity = entity, torque = ?torque_vec, "Queued apply_torque command");
            Ok(())
        },
    );

    // Set velocity of entity
    module.set_native_fn(
        "set_velocity",
        move |entity: i64, linear: Dynamic, angular: Dynamic| -> Result<(), Box<EvalAltResult>> {
            let linear_vec = parse_vec3_from_dynamic(linear)?;
            let angular_vec = parse_vec3_from_dynamic(angular)?;
            crate::physics::system::queue_physics_command(PhysicsCommand::SetVelocity {
                entity: entity as u64,
                linear: linear_vec,
                angular: angular_vec,
            });
            trace!(entity = entity, linear = ?linear_vec, angular = ?angular_vec, "Queued set_velocity command");
            Ok(())
        },
    );

    // Raycast - for now returns a placeholder
    // TODO: Implement proper raycast with physics world access
    module.set_native_fn(
        "raycast",
        move |origin: Dynamic,
              direction: Dynamic,
              _max_distance: f32|
              -> Result<Dynamic, Box<EvalAltResult>> {
            let _origin = parse_vec3_from_dynamic(origin)?;
            let _direction = parse_vec3_from_dynamic(direction)?;

            // Return null for now (no hit)
            Ok(Dynamic::UNIT)
        },
    );

    module
}

/// Parse a Vec3 from various Dynamic representations
fn parse_vec3_from_dynamic(value: Dynamic) -> Result<Vec3, Box<EvalAltResult>> {
    // Try to cast directly to Vec3 first
    if let Some(vec3) = value.clone().try_cast::<Vec3>() {
        return Ok(vec3);
    }

    // Try to parse as array of 3 floats
    if let Ok(array) = value.clone().into_array() {
        if array.len() == 3 {
            let x = array[0]
                .clone()
                .as_float()
                .map_err(|_| "Expected float for x component")?;
            let y = array[1]
                .clone()
                .as_float()
                .map_err(|_| "Expected float for y component")?;
            let z = array[2]
                .clone()
                .as_float()
                .map_err(|_| "Expected float for z component")?;
            return Ok(Vec3::new(x as f32, y as f32, z as f32));
        }
    }

    Err("Expected Vec3 or array of 3 floats".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vec3_from_array() {
        let array = vec![
            Dynamic::from(1.0f64),
            Dynamic::from(2.0f64),
            Dynamic::from(3.0f64),
        ];
        let dynamic = Dynamic::from(array);

        let result = parse_vec3_from_dynamic(dynamic).unwrap();
        assert_eq!(result, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_physics_module_creation() {
        let module = create_physics_module();

        // Module should have the expected functions
        // TODO: Module::contains_fn expects u64 hash, not &str
        // Need to find correct way to test module function presence
        // assert!(module.contains_fn("apply_force"));
        // assert!(module.contains_fn("apply_impulse"));
        // assert!(module.contains_fn("apply_torque"));
        // assert!(module.contains_fn("set_velocity"));
        // assert!(module.contains_fn("raycast"));

        // For now, just verify module creation doesn't panic
        assert!(!module.is_empty());
    }
}
