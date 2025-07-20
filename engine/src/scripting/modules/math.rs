//! Math types and functions for Rhai scripts

use crate::core::entity::Transform;
use glam::{Quat, Vec3};
use rhai::{Engine, Module};
use tracing::debug;

/// Register math types with Rhai
pub fn register_math_types(engine: &mut Engine) {
    debug!("Registering math types");

    // Register Vec3 type
    engine
        .register_type_with_name::<Vec3>("Vec3")
        .register_fn("create", |x: f64, y: f64, z: f64| {
            Vec3::new(x as f32, y as f32, z as f32)
        })
        .register_fn("zero", || Vec3::ZERO)
        .register_fn("one", || Vec3::ONE)
        .register_get("x", |v: &mut Vec3| v.x as f64)
        .register_set("x", |v: &mut Vec3, x: f64| v.x = x as f32)
        .register_get("y", |v: &mut Vec3| v.y as f64)
        .register_set("y", |v: &mut Vec3, y: f64| v.y = y as f32)
        .register_get("z", |v: &mut Vec3| v.z as f64)
        .register_set("z", |v: &mut Vec3, z: f64| v.z = z as f32)
        .register_fn("+", |a: Vec3, b: Vec3| a + b)
        .register_fn("-", |a: Vec3, b: Vec3| a - b)
        .register_fn("*", |a: Vec3, b: f64| a * b as f32)
        .register_fn("*", |a: f64, b: Vec3| b * a as f32)
        .register_fn("/", |a: Vec3, b: f64| a / b as f32)
        .register_fn("==", |a: &mut Vec3, b: Vec3| *a == b)
        .register_fn("!=", |a: &mut Vec3, b: Vec3| *a != b)
        .register_fn("length", |v: &mut Vec3| v.length() as f64)
        .register_fn("normalize", |v: &mut Vec3| v.normalize())
        .register_fn("dot", |a: &mut Vec3, b: Vec3| a.dot(b) as f64)
        .register_fn("cross", |a: &mut Vec3, b: Vec3| a.cross(b))
        .register_fn("to_string", |v: &mut Vec3| {
            format!("Vec3({}, {}, {})", v.x, v.y, v.z)
        });

    // Register Quat type
    engine
        .register_type_with_name::<Quat>("Quat")
        .register_fn("identity", || Quat::IDENTITY)
        .register_fn("from_rotation_x", |angle: f64| {
            Quat::from_rotation_x(angle as f32)
        })
        .register_fn("from_rotation_y", |angle: f64| {
            Quat::from_rotation_y(angle as f32)
        })
        .register_fn("from_rotation_z", |angle: f64| {
            Quat::from_rotation_z(angle as f32)
        })
        .register_fn("from_axis_angle", |axis: Vec3, angle: f64| {
            Quat::from_axis_angle(axis.normalize(), angle as f32)
        })
        .register_fn("*", |a: Quat, b: Quat| a * b)
        .register_fn("slerp", |a: &mut Quat, b: Quat, t: f64| {
            a.slerp(b, t as f32)
        })
        .register_fn("to_string", |q: &mut Quat| {
            format!("Quat({}, {}, {}, {})", q.x, q.y, q.z, q.w)
        });

    // Register Transform type
    engine
        .register_type_with_name::<Transform>("Transform")
        .register_fn("create", Transform::default)
        .register_fn("from_position", |pos: Vec3| Transform::from_position(pos))
        .register_get("position", |t: &mut Transform| t.position)
        .register_set("position", |t: &mut Transform, pos: Vec3| t.position = pos)
        .register_get("rotation", |t: &mut Transform| t.rotation)
        .register_set("rotation", |t: &mut Transform, rot: Quat| t.rotation = rot)
        .register_get("scale", |t: &mut Transform| t.scale)
        .register_set("scale", |t: &mut Transform, scale: Vec3| t.scale = scale)
        .register_fn("rotate_x", |t: &mut Transform, angle: f64| {
            t.rotation *= Quat::from_rotation_x(angle as f32);
        })
        .register_fn("rotate_y", |t: &mut Transform, angle: f64| {
            t.rotation *= Quat::from_rotation_y(angle as f32);
        })
        .register_fn("rotate_z", |t: &mut Transform, angle: f64| {
            t.rotation *= Quat::from_rotation_z(angle as f32);
        })
        .register_fn("rotate_vector", |t: &mut Transform, v: Vec3| t.rotation * v)
        .register_fn("looking_at", |t: &mut Transform, target: Vec3, up: Vec3| {
            t.looking_at(target, up)
        })
        .register_fn("clone", |t: &mut Transform| *t)
        .register_fn("to_string", |t: &mut Transform| {
            format!(
                "Transform(pos: {:?}, rot: {:?}, scale: {:?})",
                t.position, t.rotation, t.scale
            )
        });

    // Register math utility functions
    let mut math_module = Module::new();

    math_module.set_native_fn("rad", |degrees: f64| Ok(degrees.to_radians()));
    math_module.set_native_fn("deg", |radians: f64| Ok(radians.to_degrees()));
    math_module.set_native_fn("sin", |x: f64| Ok(x.sin()));
    math_module.set_native_fn("cos", |x: f64| Ok(x.cos()));
    math_module.set_native_fn("tan", |x: f64| Ok(x.tan()));
    math_module.set_native_fn("abs", |x: f64| Ok(x.abs()));
    math_module.set_native_fn("sqrt", |x: f64| Ok(x.sqrt()));
    math_module.set_native_fn("pow", |x: f64, y: f64| Ok(x.powf(y)));
    math_module.set_native_fn("min", |a: f64, b: f64| Ok(a.min(b)));
    math_module.set_native_fn("max", |a: f64, b: f64| Ok(a.max(b)));
    math_module.set_native_fn("clamp", |x: f64, min: f64, max: f64| Ok(x.clamp(min, max)));
    math_module.set_native_fn("lerp", |a: f64, b: f64, t: f64| Ok(a + (b - a) * t));

    // Add math constants to the math module
    math_module.set_var("PI", std::f64::consts::PI);
    math_module.set_var("TAU", std::f64::consts::TAU);
    math_module.set_var("E", std::f64::consts::E);

    engine.register_static_module("math", math_module.into());

    // Create Vec3 module with constructor functions
    let mut vec3_module = Module::new();
    vec3_module.set_native_fn("create", |x: f64, y: f64, z: f64| {
        Ok(Vec3::new(x as f32, y as f32, z as f32))
    });
    vec3_module.set_native_fn("zero", || Ok(Vec3::ZERO));
    vec3_module.set_native_fn("one", || Ok(Vec3::ONE));
    engine.register_static_module("Vec3", vec3_module.into());

    // Create Quat module with constructor functions
    let mut quat_module = Module::new();
    quat_module.set_native_fn("identity", || Ok(Quat::IDENTITY));
    quat_module.set_native_fn("from_rotation_x", |angle: f64| {
        Ok(Quat::from_rotation_x(angle as f32))
    });
    quat_module.set_native_fn("from_rotation_y", |angle: f64| {
        Ok(Quat::from_rotation_y(angle as f32))
    });
    quat_module.set_native_fn("from_rotation_z", |angle: f64| {
        Ok(Quat::from_rotation_z(angle as f32))
    });
    quat_module.set_native_fn("from_axis_angle", |axis: Vec3, angle: f64| {
        Ok(Quat::from_axis_angle(axis.normalize(), angle as f32))
    });
    engine.register_static_module("Quat", quat_module.into());

    // Create Transform module with constructor functions
    let mut transform_module = Module::new();
    transform_module.set_native_fn("create", || Ok(Transform::default()));
    transform_module.set_native_fn("from_position", |pos: Vec3| {
        Ok(Transform::from_position(pos))
    });
    engine.register_static_module("Transform", transform_module.into());

    debug!("Math types registered");
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhai::Scope;

    #[test]
    fn test_vec3_registration() {
        let mut engine = Engine::new();
        register_math_types(&mut engine);

        let result: Vec3 = engine.eval("Vec3::create(1.0, 2.0, 3.0)").unwrap();
        assert_eq!(result, Vec3::new(1.0, 2.0, 3.0));

        let result: Vec3 = engine
            .eval("Vec3::create(1.0, 0.0, 0.0) + Vec3::create(0.0, 2.0, 0.0)")
            .unwrap();
        assert_eq!(result, Vec3::new(1.0, 2.0, 0.0));

        let result: f64 = engine.eval("Vec3::create(3.0, 4.0, 0.0).length()").unwrap();
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_quat_registration() {
        let mut engine = Engine::new();
        register_math_types(&mut engine);

        let result: Quat = engine.eval("Quat::identity()").unwrap();
        assert_eq!(result, Quat::IDENTITY);

        let result: Quat = engine.eval("Quat::from_rotation_y(1.57)").unwrap();
        let expected = Quat::from_rotation_y(1.57);
        assert!((result.x - expected.x).abs() < 0.01);
        assert!((result.y - expected.y).abs() < 0.01);
        assert!((result.z - expected.z).abs() < 0.01);
        assert!((result.w - expected.w).abs() < 0.01);
    }

    #[test]
    fn test_transform_registration() {
        let mut engine = Engine::new();
        register_math_types(&mut engine);

        let mut scope = Scope::new();

        let result = engine
            .eval_with_scope::<Transform>(
                &mut scope,
                r#"
            let t = Transform::from_position(Vec3::create(1.0, 2.0, 3.0));
            t.position = Vec3::create(4.0, 5.0, 6.0);
            t
        "#,
            )
            .unwrap();

        assert_eq!(result.position, Vec3::new(4.0, 5.0, 6.0));
    }

    #[test]
    fn test_math_module() {
        let mut engine = Engine::new();
        register_math_types(&mut engine);

        let result: f64 = engine.eval("math::rad(180.0)").unwrap();
        assert!((result - std::f64::consts::PI).abs() < 0.0001);

        let result: f64 = engine.eval("math::clamp(5.0, 0.0, 3.0)").unwrap();
        assert_eq!(result, 3.0);

        let result: f64 = engine.eval("math::lerp(0.0, 10.0, 0.5)").unwrap();
        assert_eq!(result, 5.0);
    }
}
