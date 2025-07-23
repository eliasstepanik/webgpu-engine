//! Physics components for the entity system

use crate::component_system::{Component, ComponentMetadata, ComponentRegistryExt, EditorUI};
use crate::io::ComponentRegistry;
use glam::{Mat3, Vec3};
use hecs::Entity;
use serde::{Deserialize, Serialize};

/// Rigidbody component for physics simulation
#[derive(
    engine_derive::Component, engine_derive::EditorUI, Debug, Clone, Serialize, Deserialize,
)]
#[component(name = "Rigidbody")]
pub struct Rigidbody {
    /// Mass in kilograms
    #[ui(range = 0.1..1000.0, speed = 0.1, tooltip = "Mass in kg")]
    pub mass: f32,

    /// Linear damping coefficient
    #[ui(range = 0.0..10.0, speed = 0.01, tooltip = "Linear damping")]
    pub linear_damping: f32,

    /// Angular damping coefficient
    #[ui(range = 0.0..10.0, speed = 0.01, tooltip = "Angular damping")]
    pub angular_damping: f32,

    /// Linear velocity in world space
    #[ui(tooltip = "Linear velocity")]
    pub linear_velocity: Vec3,

    /// Angular velocity in world space
    #[ui(tooltip = "Angular velocity")]
    pub angular_velocity: Vec3,

    /// Inertia tensor (3x3 matrix)
    #[ui(hidden)]
    pub inertia_tensor: Mat3,

    /// Whether this body is affected by gravity
    #[ui(tooltip = "Is affected by gravity")]
    pub use_gravity: bool,

    /// Kinematic bodies are not affected by forces
    #[ui(tooltip = "Prevents all movement")]
    pub is_kinematic: bool,
}

impl Default for Rigidbody {
    fn default() -> Self {
        Self {
            mass: 1.0,
            linear_damping: 0.0,
            angular_damping: 0.0,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            inertia_tensor: Mat3::IDENTITY,
            use_gravity: true,
            is_kinematic: false,
        }
    }
}

impl Rigidbody {
    /// Create a kinematic rigidbody (not affected by forces)
    pub fn kinematic() -> Self {
        Self {
            is_kinematic: true,
            use_gravity: false,
            ..Default::default()
        }
    }

    /// Create a dynamic rigidbody with the given mass
    pub fn dynamic(mass: f32) -> Self {
        Self {
            mass,
            ..Default::default()
        }
    }

    /// Apply a force to the rigidbody
    pub fn apply_force(&mut self, force: Vec3, dt: f32) {
        if !self.is_kinematic {
            let acceleration = force / self.mass;
            self.linear_velocity += acceleration * dt;
        }
    }

    /// Apply a torque to the rigidbody
    pub fn apply_torque(&mut self, torque: Vec3, dt: f32) {
        if !self.is_kinematic {
            let angular_acceleration = self.inertia_tensor.inverse() * torque;
            self.angular_velocity += angular_acceleration * dt;
        }
    }

    /// Apply damping to velocities
    pub fn apply_damping(&mut self, dt: f32) {
        let linear_damping_factor = (1.0 - self.linear_damping * dt).max(0.0);
        let angular_damping_factor = (1.0 - self.angular_damping * dt).max(0.0);

        self.linear_velocity *= linear_damping_factor;
        self.angular_velocity *= angular_damping_factor;
    }
}

/// Collision shape types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CollisionShape {
    /// Sphere with radius
    Sphere { radius: f32 },
    /// Box with half-extents (width/2, height/2, depth/2)
    Box { half_extents: Vec3 },
    /// Capsule with radius and half-height (height is along Y axis)
    Capsule { radius: f32, half_height: f32 },
}

impl Default for CollisionShape {
    fn default() -> Self {
        CollisionShape::Box {
            half_extents: Vec3::splat(0.5),
        }
    }
}

impl CollisionShape {
    /// Calculate the inertia tensor for this shape with the given mass
    pub fn calculate_inertia(&self, mass: f32) -> Mat3 {
        match self {
            CollisionShape::Sphere { radius } => {
                let inertia = 0.4 * mass * radius * radius;
                Mat3::from_diagonal(Vec3::splat(inertia))
            }
            CollisionShape::Box { half_extents } => {
                let x = half_extents.x * 2.0;
                let y = half_extents.y * 2.0;
                let z = half_extents.z * 2.0;
                let factor = mass / 12.0;

                Mat3::from_diagonal(Vec3::new(
                    factor * (y * y + z * z),
                    factor * (x * x + z * z),
                    factor * (x * x + y * y),
                ))
            }
            CollisionShape::Capsule {
                radius,
                half_height,
            } => {
                // Approximate as cylinder for now
                let height = half_height * 2.0;
                let cylinder_mass = mass * 0.8; // Approximate 80% in cylinder
                let sphere_mass = mass * 0.2; // 20% in end caps

                let cylinder_inertia_x =
                    cylinder_mass * (3.0 * radius * radius + height * height) / 12.0;
                let cylinder_inertia_y = cylinder_mass * radius * radius / 2.0;

                let sphere_inertia = 0.4 * sphere_mass * radius * radius;
                let sphere_offset = half_height + radius * 0.5;
                let sphere_inertia_x = sphere_inertia + sphere_mass * sphere_offset * sphere_offset;

                Mat3::from_diagonal(Vec3::new(
                    cylinder_inertia_x + 2.0 * sphere_inertia_x,
                    cylinder_inertia_y + 2.0 * sphere_inertia,
                    cylinder_inertia_x + 2.0 * sphere_inertia_x,
                ))
            }
        }
    }

    /// Get the bounding sphere radius for broad phase
    pub fn bounding_radius(&self) -> f32 {
        match self {
            CollisionShape::Sphere { radius } => *radius,
            CollisionShape::Box { half_extents } => half_extents.length(),
            CollisionShape::Capsule {
                radius,
                half_height,
            } => radius + half_height,
        }
    }
}

/// Collider component for collision detection
#[derive(
    engine_derive::Component, engine_derive::EditorUI, Debug, Clone, Serialize, Deserialize, Default,
)]
#[component(name = "Collider")]
pub struct Collider {
    /// Collision shape type
    #[ui(hidden)] // Hide from metadata UI since we handle it manually
    pub shape: CollisionShape,

    /// Is this a trigger (no collision response)
    #[ui(tooltip = "Is trigger (no collision response)")]
    pub is_trigger: bool,

    /// Reference to physics material entity
    #[ui(hidden)]
    pub material_id: Option<u64>, // Store as u64 for serialization
}

impl Collider {
    /// Create a sphere collider
    pub fn sphere(radius: f32) -> Self {
        Self {
            shape: CollisionShape::Sphere { radius },
            ..Default::default()
        }
    }

    /// Create a box collider
    pub fn box_collider(half_extents: Vec3) -> Self {
        Self {
            shape: CollisionShape::Box { half_extents },
            ..Default::default()
        }
    }

    /// Create a capsule collider
    pub fn capsule(radius: f32, half_height: f32) -> Self {
        Self {
            shape: CollisionShape::Capsule {
                radius,
                half_height,
            },
            ..Default::default()
        }
    }

    /// Set this collider as a trigger
    pub fn as_trigger(mut self) -> Self {
        self.is_trigger = true;
        self
    }

    /// Set the physics material for this collider
    pub fn with_material(mut self, material_entity: Entity) -> Self {
        self.material_id = Some(material_entity.to_bits().into());
        self
    }
}

/// Physics material properties
#[derive(
    engine_derive::Component, engine_derive::EditorUI, Debug, Clone, Serialize, Deserialize,
)]
#[component(name = "PhysicsMaterial")]
pub struct PhysicsMaterial {
    /// Static friction coefficient
    #[ui(range = 0.0..1.0, speed = 0.01, tooltip = "Static friction")]
    pub static_friction: f32,

    /// Dynamic friction coefficient
    #[ui(range = 0.0..1.0, speed = 0.01, tooltip = "Dynamic friction")]
    pub dynamic_friction: f32,

    /// Restitution (bounciness) coefficient
    #[ui(range = 0.0..1.0, speed = 0.01, tooltip = "Restitution (bounciness)")]
    pub restitution: f32,
}

impl Default for PhysicsMaterial {
    fn default() -> Self {
        Self {
            static_friction: 0.6,
            dynamic_friction: 0.4,
            restitution: 0.0,
        }
    }
}

impl PhysicsMaterial {
    /// Create a bouncy material
    pub fn bouncy() -> Self {
        Self {
            restitution: 0.8,
            ..Default::default()
        }
    }

    /// Create a slippery material (ice-like)
    pub fn slippery() -> Self {
        Self {
            static_friction: 0.1,
            dynamic_friction: 0.05,
            restitution: 0.0,
        }
    }

    /// Create a sticky material (rubber-like)
    pub fn sticky() -> Self {
        Self {
            static_friction: 0.9,
            dynamic_friction: 0.8,
            restitution: 0.2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rigidbody_creation() {
        let rb = Rigidbody::default();
        assert_eq!(rb.mass, 1.0);
        assert!(rb.use_gravity);
        assert!(!rb.is_kinematic);

        let kinematic = Rigidbody::kinematic();
        assert!(kinematic.is_kinematic);
        assert!(!kinematic.use_gravity);
    }

    #[test]
    fn test_collision_shapes() {
        let sphere = CollisionShape::Sphere { radius: 1.0 };
        assert_eq!(sphere.bounding_radius(), 1.0);

        let box_shape = CollisionShape::Box {
            half_extents: Vec3::new(1.0, 2.0, 3.0),
        };
        assert!((box_shape.bounding_radius() - 3.74165).abs() < 0.001);
    }

    #[test]
    fn test_inertia_calculation() {
        let mass = 10.0;
        let sphere = CollisionShape::Sphere { radius: 1.0 };
        let inertia = sphere.calculate_inertia(mass);
        assert_eq!(inertia.x_axis.x, 4.0); // 0.4 * 10 * 1²
    }
}
