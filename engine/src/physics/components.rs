//! Physics components for the entity system
//!
//! This module provides physics components that integrate with Rapier3D-f64
//! for high-precision physics simulation supporting large world coordinates.

use crate::component_system::{Component, ComponentMetadata, ComponentRegistryExt, EditorUI};
use crate::io::component_registry::ComponentRegistry;
use engine_derive;
use glam::Vec3;
use rapier3d_f64::prelude::{ColliderHandle, RigidBodyHandle};
use serde::{Deserialize, Serialize};

/// Type of rigid body for physics simulation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RigidBodyType {
    /// Dynamic body affected by forces and collisions
    Dynamic,
    /// Fixed body that never moves (static geometry)
    Fixed,
    /// Kinematic body controlled by position
    KinematicPositionBased,
    /// Kinematic body controlled by velocity
    KinematicVelocityBased,
}

impl Default for RigidBodyType {
    fn default() -> Self {
        Self::Dynamic
    }
}

/// Rigid body component for physics simulation
#[derive(
    Debug, Clone, Serialize, Deserialize, engine_derive::Component, engine_derive::EditorUI,
)]
#[component(name = "RigidBody")]
pub struct RigidBody {
    /// Type of rigid body
    #[ui(tooltip = "Body type - Dynamic bodies are affected by forces, Fixed bodies never move")]
    pub body_type: RigidBodyType,

    /// Linear damping factor (0.0 = no damping)
    #[ui(range = 0.0..10.0, speed = 0.1, tooltip = "Linear damping - reduces linear velocity over time")]
    pub linear_damping: f32,

    /// Angular damping factor (0.0 = no damping)
    #[ui(range = 0.0..10.0, speed = 0.1, tooltip = "Angular damping - reduces angular velocity over time")]
    pub angular_damping: f32,

    /// Internal handle to the Rapier rigid body
    #[ui(hidden)]
    #[serde(skip)]
    pub handle: Option<RigidBodyHandle>,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            body_type: RigidBodyType::Dynamic,
            linear_damping: 0.5,
            angular_damping: 0.5,
            handle: None,
        }
    }
}

/// Shape of a collider
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ColliderShape {
    /// Cuboid with half-extents (half width, height, depth)
    Cuboid(Vec3),
    /// Sphere with radius
    Sphere(f32),
    /// Capsule with half-height and radius
    Capsule { half_height: f32, radius: f32 },
    /// Cylinder with half-height and radius
    Cylinder { half_height: f32, radius: f32 },
}

impl Default for ColliderShape {
    fn default() -> Self {
        Self::Cuboid(Vec3::new(0.5, 0.5, 0.5))
    }
}

/// Collider component for collision detection
#[derive(
    Debug, Clone, Serialize, Deserialize, engine_derive::Component, engine_derive::EditorUI,
)]
#[component(name = "Collider")]
pub struct Collider {
    /// Shape of the collider
    #[ui(tooltip = "Collider shape for collision detection")]
    pub shape: ColliderShape,

    /// Friction coefficient (0.0 = no friction, 1.0 = high friction)
    #[ui(range = 0.0..2.0, speed = 0.01, tooltip = "Friction coefficient - how much surfaces resist sliding")]
    pub friction: f32,

    /// Restitution coefficient (0.0 = no bounce, 1.0 = perfect bounce)
    #[ui(range = 0.0..1.0, speed = 0.01, tooltip = "Restitution (bounciness) - how much energy is preserved in collisions")]
    pub restitution: f32,

    /// Density of the collider (affects mass calculation)
    #[ui(range = 0.1..10.0, speed = 0.1, tooltip = "Density - used to calculate mass from volume")]
    pub density: f32,

    /// Whether this collider is a sensor (triggers events but doesn't cause collision response)
    #[ui(tooltip = "Sensor colliders detect overlaps but don't cause physical collision response")]
    pub is_sensor: bool,

    /// Internal handle to the Rapier collider
    #[ui(hidden)]
    #[serde(skip)]
    pub handle: Option<ColliderHandle>,
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            shape: ColliderShape::default(),
            friction: 0.5,
            restitution: 0.0,
            density: 1.0,
            is_sensor: false,
            handle: None,
        }
    }
}

impl Collider {
    /// Create a cuboid collider with the given half-extents
    pub fn cuboid(hx: f32, hy: f32, hz: f32) -> Self {
        Self {
            shape: ColliderShape::Cuboid(Vec3::new(hx, hy, hz)),
            ..Default::default()
        }
    }

    /// Create a sphere collider with the given radius
    pub fn sphere(radius: f32) -> Self {
        Self {
            shape: ColliderShape::Sphere(radius),
            ..Default::default()
        }
    }

    /// Create a capsule collider with the given half-height and radius
    pub fn capsule(half_height: f32, radius: f32) -> Self {
        Self {
            shape: ColliderShape::Capsule { half_height, radius },
            ..Default::default()
        }
    }

    /// Create a cylinder collider with the given half-height and radius
    pub fn cylinder(half_height: f32, radius: f32) -> Self {
        Self {
            shape: ColliderShape::Cylinder { half_height, radius },
            ..Default::default()
        }
    }
}

/// Physics velocity component for reading/writing velocities
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    Default,
    engine_derive::Component,
    engine_derive::EditorUI,
)]
#[component(name = "PhysicsVelocity")]
pub struct PhysicsVelocity {
    /// Linear velocity in world space
    #[ui(speed = 0.1, tooltip = "Linear velocity in world space")]
    pub linear: Vec3,

    /// Angular velocity in world space (axis-angle representation)
    #[ui(speed = 0.1, tooltip = "Angular velocity in world space")]
    pub angular: Vec3,
}

impl PhysicsVelocity {
    /// Create a new velocity with the given linear velocity
    pub fn linear(velocity: Vec3) -> Self {
        Self {
            linear: velocity,
            angular: Vec3::ZERO,
        }
    }

    /// Create a new velocity with both linear and angular components
    pub fn new(linear: Vec3, angular: Vec3) -> Self {
        Self { linear, angular }
    }
}

/// Physics mass properties component
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    engine_derive::Component,
    engine_derive::EditorUI,
)]
#[component(name = "PhysicsMass")]
pub struct PhysicsMass {
    /// Mass of the body in kilograms
    #[ui(range = 0.1..1000.0, speed = 0.1, tooltip = "Mass in kilograms")]
    pub mass: f32,

    /// Center of mass offset from the entity's transform position
    #[ui(speed = 0.01, tooltip = "Center of mass offset from transform position")]
    pub center_of_mass: Vec3,
}

impl Default for PhysicsMass {
    fn default() -> Self {
        Self {
            mass: 1.0,
            center_of_mass: Vec3::ZERO,
        }
    }
}

impl PhysicsMass {
    /// Create a new mass component with the given mass
    pub fn new(mass: f32) -> Self {
        Self {
            mass,
            center_of_mass: Vec3::ZERO,
        }
    }

    /// Create a new mass component with mass and center of mass offset
    pub fn with_center_of_mass(mass: f32, center_of_mass: Vec3) -> Self {
        Self {
            mass,
            center_of_mass,
        }
    }
}