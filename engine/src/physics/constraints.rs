//! Constraint system for AVBD physics

use crate::physics::avbd_solver::{Mat6, RigidbodyData};
use crate::physics::collision::Contact;
use crate::physics::components::PhysicsMaterial;
use glam::{Mat3, Vec3};
use std::fmt::Debug;
use tracing::trace;

/// Trait for constraints in the AVBD system
pub trait Constraint: Send + Sync + Debug {
    /// Get the indices of bodies affected by this constraint
    fn body_indices(&self) -> Vec<usize>;

    /// Check if this constraint affects a specific body
    fn affects_body(&self, body_idx: usize) -> bool {
        self.body_indices().contains(&body_idx)
    }

    /// Evaluate the constraint function C(x)
    fn evaluate(&self, bodies: &[RigidbodyData]) -> Vec3;

    /// Compute force and Hessian contribution for a specific body
    fn compute_force_and_hessian(
        &self,
        body_idx: usize,
        bodies: &[RigidbodyData],
    ) -> ((Vec3, Vec3), Mat6);

    /// Warm-start the constraint (scale lambda and k by gamma)
    fn warm_start(&mut self, gamma: f32);

    /// Update dual variables (lambda and k)
    fn update_dual_variables(&mut self, bodies: &[RigidbodyData], beta: f32, alpha: f32);

    /// Get contact information if this is a contact constraint
    fn get_contact_info(&self) -> Option<(Vec3, f32)> {
        None // Default implementation returns None
    }

    /// Check if constraint involves static bodies that need special handling
    fn has_static_body(&self) -> bool {
        false // Default implementation returns false
    }
}

/// Base constraint data for AVBD
#[derive(Debug, Clone)]
pub struct ConstraintData {
    /// Lagrange multipliers (dual variables)
    pub lambda: Vec3,
    /// Penalty parameters (stiffness)
    pub k: Vec3,
    /// Minimum force bounds
    pub lambda_min: Vec3,
    /// Maximum force bounds
    pub lambda_max: Vec3,
    /// Target stiffness (for soft constraints)
    pub k_target: Vec3,
}

impl ConstraintData {
    /// Create constraint data for a hard constraint
    pub fn hard(lambda_min: Vec3, lambda_max: Vec3, k_start: f32) -> Self {
        Self {
            lambda: Vec3::ZERO,
            k: Vec3::splat(k_start),
            lambda_min,
            lambda_max,
            k_target: Vec3::splat(f32::INFINITY),
        }
    }

    /// Create constraint data for a soft constraint
    pub fn soft(k_target: f32) -> Self {
        Self {
            lambda: Vec3::ZERO,
            k: Vec3::splat(k_target),
            lambda_min: Vec3::splat(f32::NEG_INFINITY),
            lambda_max: Vec3::splat(f32::INFINITY),
            k_target: Vec3::splat(k_target),
        }
    }

    /// Warm-start by scaling lambda and k
    pub fn warm_start(&mut self, gamma: f32) {
        self.lambda *= gamma;
        self.k *= gamma;
        // Ensure k doesn't go below some minimum
        self.k = self.k.max(Vec3::splat(10.0));
    }

    /// Update dual variables for one component
    fn update_component(&mut self, c: f32, beta: f32, alpha: f32, idx: usize) {
        let is_hard = self.k_target[idx].is_infinite();

        if is_hard {
            // Hard constraint: update lambda and possibly k
            let new_lambda = self.k[idx] * c + self.lambda[idx];
            self.lambda[idx] = new_lambda.clamp(self.lambda_min[idx], self.lambda_max[idx]);

            // Only increase k if lambda is not at bounds
            if self.lambda[idx] > self.lambda_min[idx] && self.lambda[idx] < self.lambda_max[idx] {
                self.k[idx] += beta * c.abs();
            }
        } else {
            // Soft constraint: k is fixed, lambda tracks accumulated error
            self.lambda[idx] = alpha * (self.k[idx] * c + self.lambda[idx]);
            self.k[idx] = self.k_target[idx];
        }
    }
}

/// Contact constraint for collision response
#[derive(Debug)]
pub struct ContactConstraint {
    /// Indices of the two bodies
    pub body_a: usize,
    pub body_b: usize,
    /// Local contact point on body A
    pub local_point_a: Vec3,
    /// Local contact point on body B
    pub local_point_b: Vec3,
    /// Contact normal in world space (from B to A)
    pub normal: Vec3,
    /// Contact tangent for friction
    pub tangent: Vec3,
    /// Contact bitangent for friction
    pub bitangent: Vec3,
    /// Friction coefficients
    pub friction: f32,
    /// Restitution coefficient
    pub restitution: f32,
    /// Constraint data
    pub data: ConstraintData,
    /// Target relative velocity (for restitution)
    pub target_velocity: f32,
}

impl ContactConstraint {
    /// Create a contact constraint with optional bodies (for static colliders)
    pub fn new_with_optional_bodies(
        contact: Contact,
        body_a_idx: Option<usize>,
        body_b_idx: Option<usize>,
        bodies: &[RigidbodyData],
        material: Option<&PhysicsMaterial>,
        _dt: f32,
    ) -> Self {
        // Handle static bodies by creating dummy indices
        let (body_a_idx, body_b_idx) = match (body_a_idx, body_b_idx) {
            (Some(a), Some(b)) => (a, b),
            (Some(a), None) => (a, usize::MAX),
            (None, Some(b)) => (usize::MAX, b),
            (None, None) => panic!("At least one body must have a rigidbody"),
        };

        // Use the regular new method
        Self::new(contact, body_a_idx, body_b_idx, bodies, material, _dt)
    }

    /// Create a contact constraint from collision data
    pub fn new(
        contact: Contact,
        body_a_idx: usize,
        body_b_idx: usize,
        bodies: &[RigidbodyData],
        material: Option<&PhysicsMaterial>,
        _dt: f32,
    ) -> Self {
        // Handle static bodies
        let (local_point_a, local_point_b) = if body_a_idx == usize::MAX {
            // Body A is static
            let body_b = &bodies[body_b_idx];
            let local_point_b = body_b.rotation.conjugate() * (contact.position - body_b.position);
            let local_point_a = contact.position; // For static body, use world position
            (local_point_a, local_point_b)
        } else if body_b_idx == usize::MAX {
            // Body B is static
            let body_a = &bodies[body_a_idx];
            let local_point_a = body_a.rotation.conjugate() * (contact.position - body_a.position);
            let local_point_b = contact.position; // For static body, use world position
            (local_point_a, local_point_b)
        } else {
            // Both are dynamic
            let body_a = &bodies[body_a_idx];
            let body_b = &bodies[body_b_idx];
            let local_point_a = body_a.rotation.conjugate() * (contact.position - body_a.position);
            let local_point_b = body_b.rotation.conjugate() * (contact.position - body_b.position);
            (local_point_a, local_point_b)
        };

        // Get material properties
        let (friction, restitution) = if let Some(mat) = material {
            (mat.dynamic_friction, mat.restitution)
        } else {
            (0.4, 0.0) // Default values
        };

        // Compute target velocity for restitution
        let relative_velocity = if body_a_idx == usize::MAX {
            // Body A is static - only body B velocity matters
            let body_b = &bodies[body_b_idx];
            let r_b = contact.position - body_b.position;
            -(body_b.linear_velocity + body_b.angular_velocity.cross(r_b)) // Negate because we want velocity of A relative to B
        } else if body_b_idx == usize::MAX {
            // Body B is static - only body A velocity matters
            let body_a = &bodies[body_a_idx];
            let r_a = contact.position - body_a.position;
            body_a.linear_velocity + body_a.angular_velocity.cross(r_a)
        } else {
            // Both are dynamic
            let body_a = &bodies[body_a_idx];
            let body_b = &bodies[body_b_idx];
            compute_relative_velocity(body_a, body_b, contact.position)
        };

        let normal_velocity = relative_velocity.dot(contact.normal);
        let target_velocity = if normal_velocity < -0.1 {
            -restitution * normal_velocity
        } else {
            0.0
        };

        // NGS position correction is handled separately, no Baumgarte bias needed

        // Set up constraint bounds
        // Normal: can only push (lambda >= 0)
        // Tangent/Bitangent: bounded by friction cone
        let lambda_min = Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, 0.0);
        let lambda_max = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);

        // Create constraint data with moderate stiffness for contacts
        let data = ConstraintData::hard(lambda_min, lambda_max, 1000.0);

        Self {
            body_a: body_a_idx,
            body_b: body_b_idx,
            local_point_a,
            local_point_b,
            normal: contact.normal,
            tangent: contact.tangent,
            bitangent: contact.bitangent,
            friction,
            restitution,
            data,
            target_velocity,
        }
    }
}

impl Constraint for ContactConstraint {
    fn body_indices(&self) -> Vec<usize> {
        let mut indices = Vec::new();
        if self.body_a != usize::MAX {
            indices.push(self.body_a);
        }
        if self.body_b != usize::MAX {
            indices.push(self.body_b);
        }
        indices
    }

    fn evaluate(&self, bodies: &[RigidbodyData]) -> Vec3 {
        // Handle static bodies
        let (world_point_a, world_point_b) = if self.body_a == usize::MAX {
            // Body A is static
            let body_b = &bodies[self.body_b];
            let world_point_b = body_b.position + body_b.rotation * self.local_point_b;
            (self.local_point_a, world_point_b) // local_point_a is already in world space for static
        } else if self.body_b == usize::MAX {
            // Body B is static
            let body_a = &bodies[self.body_a];
            let world_point_a = body_a.position + body_a.rotation * self.local_point_a;
            (world_point_a, self.local_point_b) // local_point_b is already in world space for static
        } else {
            // Both are dynamic
            let body_a = &bodies[self.body_a];
            let body_b = &bodies[self.body_b];
            let world_point_a = body_a.position + body_a.rotation * self.local_point_a;
            let world_point_b = body_b.position + body_b.rotation * self.local_point_b;
            (world_point_a, world_point_b)
        };

        // Contact separation
        let separation = world_point_a - world_point_b;

        // Project onto contact basis
        let c = Vec3::new(
            separation.dot(self.tangent),
            separation.dot(self.bitangent),
            separation.dot(self.normal),
        );

        // Debug log penetration
        if c.z < -0.001 {
            trace!(
                "Contact penetration: {:?} between bodies {} and {}",
                c.z,
                self.body_a,
                self.body_b
            );
        }

        c
    }

    fn compute_force_and_hessian(
        &self,
        body_idx: usize,
        bodies: &[RigidbodyData],
    ) -> ((Vec3, Vec3), Mat6) {
        // Static bodies should never call this method
        if body_idx == usize::MAX {
            return ((Vec3::ZERO, Vec3::ZERO), Mat6::ZERO);
        }

        let is_body_a = body_idx == self.body_a;
        let sign = if is_body_a { 1.0 } else { -1.0 };

        let body = &bodies[body_idx];
        let world_point = body.position
            + body.rotation
                * if is_body_a {
                    self.local_point_a
                } else {
                    self.local_point_b
                };
        let r = world_point - body.position;

        // Constraint Jacobian
        let j_linear = Mat3::from_cols(self.tangent, self.bitangent, self.normal) * sign;
        let j_angular = Mat3::from_cols(
            r.cross(self.tangent) * sign,
            r.cross(self.bitangent) * sign,
            r.cross(self.normal) * sign,
        );

        // Evaluate constraint
        let c = self.evaluate(bodies);

        // Clamp forces based on friction
        let mut lambda_eff = self.data.lambda + self.data.k * c;

        // Friction cone constraint
        let normal_force = lambda_eff.z.max(0.0);
        let max_friction = self.friction * normal_force;
        let friction_force = Vec3::new(lambda_eff.x, lambda_eff.y, 0.0);
        if friction_force.length() > max_friction && max_friction > 0.0 {
            let friction_dir = friction_force.normalize();
            lambda_eff.x = friction_dir.x * max_friction;
            lambda_eff.y = friction_dir.y * max_friction;
        }
        lambda_eff.z = lambda_eff.z.max(0.0);

        // Force
        let force = j_linear * lambda_eff;
        let torque = j_angular * lambda_eff;

        // Hessian approximation
        let k_diag = Mat3::from_diagonal(self.data.k);
        let h_linear = j_linear * k_diag * j_linear.transpose();
        let h_angular = j_angular * k_diag * j_angular.transpose();
        let h_coupling = j_linear * k_diag * j_angular.transpose();

        let hessian = Mat6::from_blocks(h_linear, h_coupling, h_coupling.transpose(), h_angular);

        ((force, torque), hessian)
    }

    fn warm_start(&mut self, gamma: f32) {
        self.data.warm_start(gamma);
    }

    fn update_dual_variables(&mut self, bodies: &[RigidbodyData], beta: f32, alpha: f32) {
        let c = self.evaluate(bodies);

        // Update normal component
        self.data.update_component(c.z, beta, alpha, 2);

        // Update friction components with cone constraint
        let normal_force = self.data.lambda.z.max(0.0);
        let max_friction = self.friction * normal_force;

        // Tentative friction update
        self.data.update_component(c.x, beta, alpha, 0);
        self.data.update_component(c.y, beta, alpha, 1);

        // Project onto friction cone
        let friction_force = Vec3::new(self.data.lambda.x, self.data.lambda.y, 0.0);
        if friction_force.length() > max_friction && max_friction > 0.0 {
            let friction_dir = friction_force.normalize();
            self.data.lambda.x = friction_dir.x * max_friction;
            self.data.lambda.y = friction_dir.y * max_friction;
        }
    }

    fn get_contact_info(&self) -> Option<(Vec3, f32)> {
        // Return the contact normal and penetration depth
        Some((self.normal, self.data.lambda.z))
    }

    fn has_static_body(&self) -> bool {
        self.body_a == usize::MAX || self.body_b == usize::MAX
    }
}

/// Ball joint constraint (3 DOF rotation)
#[derive(Debug)]
pub struct BallJoint {
    /// Indices of the two bodies
    pub body_a: usize,
    pub body_b: usize,
    /// Local anchor point on body A
    pub local_anchor_a: Vec3,
    /// Local anchor point on body B
    pub local_anchor_b: Vec3,
    /// Constraint data
    pub data: ConstraintData,
}

impl BallJoint {
    /// Create a new ball joint
    pub fn new(
        body_a_idx: usize,
        body_b_idx: usize,
        world_anchor: Vec3,
        bodies: &[RigidbodyData],
    ) -> Self {
        let body_a = &bodies[body_a_idx];
        let body_b = &bodies[body_b_idx];

        // Transform anchor to local space
        let local_anchor_a = body_a.rotation.conjugate() * (world_anchor - body_a.position);
        let local_anchor_b = body_b.rotation.conjugate() * (world_anchor - body_b.position);

        // Hard constraint with no bounds
        let data = ConstraintData::hard(
            Vec3::splat(f32::NEG_INFINITY),
            Vec3::splat(f32::INFINITY),
            10000.0,
        );

        Self {
            body_a: body_a_idx,
            body_b: body_b_idx,
            local_anchor_a,
            local_anchor_b,
            data,
        }
    }
}

impl Constraint for BallJoint {
    fn body_indices(&self) -> Vec<usize> {
        vec![self.body_a, self.body_b]
    }

    fn evaluate(&self, bodies: &[RigidbodyData]) -> Vec3 {
        let body_a = &bodies[self.body_a];
        let body_b = &bodies[self.body_b];

        // World space anchor points
        let world_anchor_a = body_a.position + body_a.rotation * self.local_anchor_a;
        let world_anchor_b = body_b.position + body_b.rotation * self.local_anchor_b;

        // Constraint: anchors should coincide
        world_anchor_a - world_anchor_b
    }

    fn compute_force_and_hessian(
        &self,
        body_idx: usize,
        bodies: &[RigidbodyData],
    ) -> ((Vec3, Vec3), Mat6) {
        let is_body_a = body_idx == self.body_a;
        let sign = if is_body_a { 1.0 } else { -1.0 };

        let body = &bodies[body_idx];
        let local_anchor = if is_body_a {
            self.local_anchor_a
        } else {
            self.local_anchor_b
        };
        let r = body.rotation * local_anchor;

        // Jacobian
        let _j_linear = Mat3::IDENTITY * sign;
        let j_angular = skew_matrix(r) * sign;

        // Evaluate constraint
        let c = self.evaluate(bodies);
        let lambda_eff = self.data.lambda + self.data.k * c;

        // Force
        let force = lambda_eff * sign;
        let torque = r.cross(lambda_eff) * sign;

        // Hessian
        let k_mat = Mat3::from_diagonal(self.data.k);
        let h_linear = k_mat * sign * sign;
        let h_angular = j_angular.transpose() * k_mat * j_angular;
        let h_coupling = k_mat * j_angular * sign;

        let hessian = Mat6::from_blocks(h_linear, h_coupling, h_coupling.transpose(), h_angular);

        ((force, torque), hessian)
    }

    fn warm_start(&mut self, gamma: f32) {
        self.data.warm_start(gamma);
    }

    fn update_dual_variables(&mut self, bodies: &[RigidbodyData], beta: f32, alpha: f32) {
        let c = self.evaluate(bodies);
        self.data.update_component(c.x, beta, alpha, 0);
        self.data.update_component(c.y, beta, alpha, 1);
        self.data.update_component(c.z, beta, alpha, 2);
    }
}

/// Compute relative velocity at a world point
fn compute_relative_velocity(
    body_a: &RigidbodyData,
    body_b: &RigidbodyData,
    world_point: Vec3,
) -> Vec3 {
    let r_a = world_point - body_a.position;
    let r_b = world_point - body_b.position;

    let vel_a = body_a.linear_velocity + body_a.angular_velocity.cross(r_a);
    let vel_b = body_b.linear_velocity + body_b.angular_velocity.cross(r_b);

    vel_a - vel_b
}

/// Create skew-symmetric matrix for cross product
fn skew_matrix(v: Vec3) -> Mat3 {
    Mat3::from_cols(
        Vec3::new(0.0, v.z, -v.y),
        Vec3::new(-v.z, 0.0, v.x),
        Vec3::new(v.y, -v.x, 0.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_data_update() {
        let mut data = ConstraintData::hard(
            Vec3::new(0.0, f32::NEG_INFINITY, f32::NEG_INFINITY),
            Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            100.0,
        );

        // Test hard constraint update
        data.update_component(0.1, 10.0, 0.95, 0);
        assert!(data.lambda.x >= 0.0); // Should be clamped to minimum

        // Test soft constraint
        let mut soft_data = ConstraintData::soft(1000.0);
        soft_data.update_component(0.1, 10.0, 0.95, 0);
        assert_eq!(soft_data.k.x, 1000.0); // k should remain fixed
    }
}
