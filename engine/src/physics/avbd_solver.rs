//! Augmented Vertex Block Descent (AVBD) solver implementation

use crate::physics::constraints::Constraint;
use glam::{Mat3, Quat, Vec3};
use hecs::Entity;
use rayon::prelude::*;
use tracing::{debug, trace};

/// Configuration for the AVBD solver
#[derive(Debug, Clone)]
pub struct AVBDConfig {
    /// Number of solver iterations per frame
    pub iterations: u32,
    /// Stiffness ramping parameter (β)
    pub beta: f32,
    /// Error correction factor (α)
    pub alpha: f32,
    /// Warm-start decay factor (γ)
    pub gamma: f32,
    /// Initial stiffness value
    pub k_start: f32,
    /// Gravity acceleration
    pub gravity: Vec3,
}

impl Default for AVBDConfig {
    fn default() -> Self {
        Self {
            iterations: 4,   // Much lower for performance
            beta: 15.0,      // Higher for faster convergence
            alpha: 0.9,      // Lower for stability
            gamma: 0.8,      // Less warm-starting
            k_start: 1000.0, // Lower stiffness
            gravity: Vec3::new(0.0, -9.81, 0.0),
        }
    }
}

/// Data for a single rigid body in the solver
#[derive(Debug, Clone)]
pub struct RigidbodyData {
    pub entity: Entity,
    /// Position (center of mass)
    pub position: Vec3,
    /// Rotation quaternion
    pub rotation: Quat,
    /// Linear velocity
    pub linear_velocity: Vec3,
    /// Angular velocity
    pub angular_velocity: Vec3,
    /// Mass (0 for kinematic bodies)
    pub mass: f32,
    /// Inverse mass (0 for kinematic bodies)
    pub inv_mass: f32,
    /// Inertia tensor in body space
    pub inertia_local: Mat3,
    /// Inverse inertia tensor in world space
    pub inv_inertia_world: Mat3,
    /// Whether body uses gravity
    pub use_gravity: bool,
    /// Is this a kinematic body
    pub is_kinematic: bool,
    /// Linear damping factor
    pub linear_damping: f32,
    /// Angular damping factor
    pub angular_damping: f32,
    /// Inertial position (y in AVBD paper)
    pub inertial_position: Vec3,
    /// Inertial rotation
    pub inertial_rotation: Quat,
    /// Vertex color for parallelization
    pub color: u32,
}

impl RigidbodyData {
    /// Create rigidbody data from components
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        entity: Entity,
        position: Vec3,
        rotation: Quat,
        linear_velocity: Vec3,
        angular_velocity: Vec3,
        mass: f32,
        inertia_local: Mat3,
        use_gravity: bool,
        is_kinematic: bool,
        linear_damping: f32,
        angular_damping: f32,
    ) -> Self {
        let inv_mass = if is_kinematic || mass <= 0.0 {
            0.0
        } else {
            1.0 / mass
        };
        let inv_inertia_world = if is_kinematic || mass <= 0.0 {
            Mat3::ZERO
        } else {
            let rot_mat = Mat3::from_quat(rotation);
            rot_mat * inertia_local.inverse() * rot_mat.transpose()
        };

        Self {
            entity,
            position,
            rotation,
            linear_velocity,
            angular_velocity,
            mass,
            inv_mass,
            inertia_local,
            inv_inertia_world,
            use_gravity,
            is_kinematic,
            linear_damping,
            angular_damping,
            inertial_position: position,
            inertial_rotation: rotation,
            color: 0,
        }
    }

    /// Update inertial position and rotation for the next timestep
    pub fn update_inertial(&mut self, dt: f32, gravity: Vec3) {
        if self.is_kinematic {
            self.inertial_position = self.position;
            self.inertial_rotation = self.rotation;
            return;
        }

        // Apply exponential damping to velocities
        self.linear_velocity *= (1.0 - self.linear_damping).powf(dt);
        self.angular_velocity *= (1.0 - self.angular_damping).powf(dt);

        // y = x + dt*v + 0.5*dt²*a_ext (standard physics formula)
        let external_acceleration = if self.use_gravity {
            gravity
        } else {
            Vec3::ZERO
        };
        self.inertial_position =
            self.position + self.linear_velocity * dt + 0.5 * external_acceleration * dt * dt;

        // Update rotation: q' = q + 0.5 * dt * ω * q
        let omega_quat = Quat::from_xyzw(
            self.angular_velocity.x,
            self.angular_velocity.y,
            self.angular_velocity.z,
            0.0,
        );
        let dq = (omega_quat * self.rotation) * (0.5 * dt);
        self.inertial_rotation = (self.rotation + dq).normalize();
    }

    /// Update world-space inverse inertia tensor
    pub fn update_inv_inertia_world(&mut self) {
        if self.is_kinematic || self.inv_mass == 0.0 {
            self.inv_inertia_world = Mat3::ZERO;
        } else {
            let rot_mat = Mat3::from_quat(self.rotation);
            self.inv_inertia_world = rot_mat * self.inertia_local.inverse() * rot_mat.transpose();
        }
    }

    /// Get generalized mass matrix (6x6)
    pub fn mass_matrix(&self) -> Mat6 {
        if self.is_kinematic {
            Mat6::ZERO
        } else {
            Mat6::from_blocks(
                Mat3::from_diagonal(Vec3::splat(self.mass)),
                Mat3::ZERO,
                Mat3::ZERO,
                self.inertia_local,
            )
        }
    }

    /// Get generalized inverse mass matrix (6x6)
    pub fn inv_mass_matrix(&self) -> Mat6 {
        if self.is_kinematic {
            Mat6::ZERO
        } else {
            Mat6::from_blocks(
                Mat3::from_diagonal(Vec3::splat(self.inv_mass)),
                Mat3::ZERO,
                Mat3::ZERO,
                self.inv_inertia_world,
            )
        }
    }
}

/// Vertex coloring for parallelization
#[derive(Debug, Clone)]
pub struct VertexColoring {
    /// Maximum color index
    pub max_color: u32,
    /// Colors per vertex
    pub colors: Vec<u32>,
}

impl VertexColoring {
    /// Create a simple sequential coloring (no parallelization)
    pub fn sequential(count: usize) -> Self {
        Self {
            max_color: 0,
            colors: vec![0; count],
        }
    }

    /// Create a greedy graph coloring
    pub fn greedy(adjacency: &[Vec<usize>]) -> Self {
        let n = adjacency.len();
        let mut colors = vec![0u32; n];
        let mut max_color = 0;

        for i in 0..n {
            let mut used_colors = vec![false; n];

            // Mark colors used by neighbors
            for &neighbor in &adjacency[i] {
                if neighbor < i {
                    used_colors[colors[neighbor] as usize] = true;
                }
            }

            // Find first available color
            let mut color = 0;
            while color < n && used_colors[color] {
                color += 1;
            }

            colors[i] = color as u32;
            max_color = max_color.max(color as u32);
        }

        Self { max_color, colors }
    }
}

/// 6x6 matrix for rigid body dynamics
#[derive(Debug, Clone, Copy)]
pub struct Mat6 {
    // Stored as four 3x3 blocks:
    // [ A  B ]
    // [ C  D ]
    pub a: Mat3,
    pub b: Mat3,
    pub c: Mat3,
    pub d: Mat3,
}

impl Mat6 {
    pub const ZERO: Self = Self {
        a: Mat3::ZERO,
        b: Mat3::ZERO,
        c: Mat3::ZERO,
        d: Mat3::ZERO,
    };

    pub const IDENTITY: Self = Self {
        a: Mat3::IDENTITY,
        b: Mat3::ZERO,
        c: Mat3::ZERO,
        d: Mat3::IDENTITY,
    };

    /// Create from blocks
    pub fn from_blocks(a: Mat3, b: Mat3, c: Mat3, d: Mat3) -> Self {
        Self { a, b, c, d }
    }

    /// Multiply by a 6D vector (represented as two Vec3)
    pub fn mul_vec6(&self, v_linear: Vec3, v_angular: Vec3) -> (Vec3, Vec3) {
        (
            self.a * v_linear + self.b * v_angular,
            self.c * v_linear + self.d * v_angular,
        )
    }

    /// Add another Mat6
    pub fn add(&self, other: &Mat6) -> Mat6 {
        Mat6 {
            a: self.a + other.a,
            b: self.b + other.b,
            c: self.c + other.c,
            d: self.d + other.d,
        }
    }

    /// Scale by a scalar
    pub fn scale(&self, s: f32) -> Mat6 {
        Mat6 {
            a: self.a * s,
            b: self.b * s,
            c: self.c * s,
            d: self.d * s,
        }
    }
}

/// AVBD solver
pub struct AVBDSolver {
    /// Solver configuration
    pub config: AVBDConfig,
    /// Active constraints
    pub constraints: Vec<Box<dyn Constraint>>,
    /// Vertex coloring for parallelization
    vertex_coloring: VertexColoring,
    /// Position correction iterations (from PhysicsConfig)
    pub position_iterations: u32,
    /// Contact slop (from PhysicsConfig)
    pub contact_slop: f32,
    /// Position correction rate (from PhysicsConfig)
    pub position_correction_rate: f32,
    /// Maximum linear velocity
    pub max_linear_velocity: f32,
    /// Maximum angular velocity
    pub max_angular_velocity: f32,
}

impl AVBDSolver {
    /// Create a new AVBD solver
    pub fn new(config: AVBDConfig) -> Self {
        Self {
            config,
            constraints: Vec::new(),
            vertex_coloring: VertexColoring::sequential(0),
            position_iterations: 4,
            contact_slop: 0.004,
            position_correction_rate: 0.8,
            max_linear_velocity: 100.0,
            max_angular_velocity: 100.0,
        }
    }

    /// Create with physics config
    pub fn with_physics_config(
        avbd_config: AVBDConfig,
        physics_config: &crate::physics::PhysicsConfig,
    ) -> Self {
        Self {
            config: avbd_config,
            constraints: Vec::new(),
            vertex_coloring: VertexColoring::sequential(0),
            position_iterations: physics_config.position_iterations,
            contact_slop: physics_config.contact_slop,
            position_correction_rate: physics_config.position_correction_rate,
            max_linear_velocity: physics_config.max_linear_velocity,
            max_angular_velocity: physics_config.max_angular_velocity,
        }
    }

    /// Update vertex coloring based on current constraints
    pub fn update_coloring(&mut self, bodies: &[RigidbodyData]) {
        // Build adjacency list
        let mut adjacency = vec![Vec::new(); bodies.len()];

        for constraint in &self.constraints {
            let bodies_indices = constraint.body_indices();
            if bodies_indices.len() == 2 {
                let (i, j) = (bodies_indices[0], bodies_indices[1]);
                adjacency[i].push(j);
                adjacency[j].push(i);
            }
        }

        // Compute greedy coloring
        self.vertex_coloring = VertexColoring::greedy(&adjacency);

        debug!(
            "Updated vertex coloring: {} colors for {} bodies",
            self.vertex_coloring.max_color + 1,
            bodies.len()
        );
    }

    /// Perform one AVBD timestep
    pub fn step(&mut self, bodies: &mut [RigidbodyData], dt: f32) {
        // 1. Update inertial positions
        bodies.par_iter_mut().for_each(|body| {
            body.update_inertial(dt, self.config.gravity);
            body.update_inv_inertia_world();
        });

        // 2. Warm-start constraints
        self.constraints.par_iter_mut().for_each(|constraint| {
            constraint.warm_start(self.config.gamma);
        });

        // 3. Main solver loop
        for iteration in 0..self.config.iterations {
            trace!(
                "AVBD iteration {}/{}",
                iteration + 1,
                self.config.iterations
            );

            // Primal update (per color)
            for color in 0..=self.vertex_coloring.max_color {
                // First, compute forces for all bodies of this color
                let body_updates: Vec<_> = (0..bodies.len())
                    .into_par_iter()
                    .filter(|i| self.vertex_coloring.colors[*i] == color)
                    .filter_map(|body_idx| {
                        let body = &bodies[body_idx];
                        if body.is_kinematic {
                            return None;
                        }

                        // Compute forces and Hessian
                        let (f, h) = self.compute_forces_and_hessian(body_idx, body, bodies, dt);

                        // Solve for position/rotation update
                        let (delta_pos, delta_rot) = self.solve_system(h, f);

                        Some((body_idx, delta_pos, delta_rot))
                    })
                    .collect();

                // Then apply updates
                for (body_idx, delta_pos, delta_rot) in body_updates {
                    let body = &mut bodies[body_idx];

                    // Debug log large updates
                    if delta_pos.length() > 0.01 || delta_rot.length() > 0.01 {
                        trace!(
                            "Body {} update: delta_pos={:?}, delta_rot={:?}",
                            body_idx,
                            delta_pos,
                            delta_rot
                        );
                    }

                    // Update position
                    body.position += delta_pos;

                    // Update rotation using special quaternion addition
                    let omega = delta_rot;
                    let dq = Quat::from_xyzw(omega.x, omega.y, omega.z, 0.0) * body.rotation;
                    body.rotation = (body.rotation + dq * 0.5).normalize();

                    // Update world inertia
                    body.update_inv_inertia_world();
                }
            }

            // Dual update (parallel over constraints)
            self.constraints.par_iter_mut().for_each(|constraint| {
                constraint.update_dual_variables(bodies, self.config.beta, self.config.alpha);
            });
        }

        // 4. Apply position correction (NGS)
        self.apply_position_correction(bodies);

        // 5. Update velocities
        bodies.par_iter_mut().for_each(|body| {
            if !body.is_kinematic {
                // Store old position for velocity calculation
                let old_position = body.inertial_position
                    - body.linear_velocity * dt
                    - 0.5 * self.config.gravity * dt * dt;

                // Linear velocity = (new_position - old_position) / dt
                body.linear_velocity = (body.position - old_position) / dt;
                
                // Clamp linear velocity to prevent tunneling
                if body.linear_velocity.length() > self.max_linear_velocity {
                    body.linear_velocity = body.linear_velocity.normalize() * self.max_linear_velocity;
                }

                // Angular velocity from quaternion difference
                let dq = body.rotation * body.inertial_rotation.conjugate();
                body.angular_velocity = 2.0 * Vec3::new(dq.x, dq.y, dq.z) / dt;
                
                // Clamp angular velocity
                if body.angular_velocity.length() > self.max_angular_velocity {
                    body.angular_velocity = body.angular_velocity.normalize() * self.max_angular_velocity;
                }
            }
        });
    }

    /// Compute forces and Hessian for a body
    fn compute_forces_and_hessian(
        &self,
        body_idx: usize,
        body: &RigidbodyData,
        all_bodies: &[RigidbodyData],
        dt: f32,
    ) -> ((Vec3, Vec3), Mat6) {
        // Inertial force: f = -(M/dt²)(x - y)
        let mass_over_dt2 = body.mass / (dt * dt);
        let inertia_over_dt2 = body.inertia_local / (dt * dt);

        let f_inertial = -mass_over_dt2 * (body.position - body.inertial_position);
        let tau_inertial = {
            // Compute rotation difference
            let dq = body.rotation * body.inertial_rotation.conjugate();
            let angle_axis = 2.0 * Vec3::new(dq.x, dq.y, dq.z);
            -inertia_over_dt2 * angle_axis
        };

        // Start with inertial terms
        let mut f = (f_inertial, tau_inertial);
        let mut h = Mat6::from_blocks(
            Mat3::from_diagonal(Vec3::splat(mass_over_dt2)),
            Mat3::ZERO,
            Mat3::ZERO,
            inertia_over_dt2,
        );

        // Add constraint forces
        for constraint in &self.constraints {
            if constraint.affects_body(body_idx) {
                let (f_constraint, h_constraint) =
                    constraint.compute_force_and_hessian(body_idx, all_bodies);
                // Subtract constraint forces (they push against constraint violation)
                f.0 -= f_constraint.0;
                f.1 -= f_constraint.1;
                h = h.add(&h_constraint);
            }
        }

        (f, h)
    }

    /// Solve the 6x6 linear system using LDLT decomposition
    fn solve_system(&self, h: Mat6, f: (Vec3, Vec3)) -> (Vec3, Vec3) {
        // Use block matrix inversion for 6x6 system
        // [A B] [x1]   [f1]
        // [C D] [x2] = [f2]

        // Check if we can invert the diagonal blocks
        let a_inv = if h.a.determinant().abs() > 1e-6 {
            h.a.inverse()
        } else {
            return (Vec3::ZERO, Vec3::ZERO);
        };

        let d_inv = if h.d.determinant().abs() > 1e-6 {
            h.d.inverse()
        } else {
            return (a_inv * f.0, Vec3::ZERO);
        };

        // Schur complement method
        // S = D - C * A^-1 * B
        let schur = h.d - h.c * a_inv * h.b;
        let schur_inv = if schur.determinant().abs() > 1e-6 {
            schur.inverse()
        } else {
            // Fall back to diagonal approximation
            return (a_inv * f.0, d_inv * f.1);
        };

        // x2 = S^-1 * (f2 - C * A^-1 * f1)
        let delta_rot = schur_inv * (f.1 - h.c * a_inv * f.0);

        // x1 = A^-1 * (f1 - B * x2)
        let delta_pos = a_inv * (f.0 - h.b * delta_rot);

        (delta_pos, delta_rot)
    }

    /// Apply NGS position correction for contact constraints
    fn apply_position_correction(&self, bodies: &mut [RigidbodyData]) {
        for _ in 0..self.position_iterations {
            // Process each contact constraint
            for constraint in &self.constraints {
                // Check if this is a contact constraint by evaluating it
                let c = constraint.evaluate(bodies);

                // Only process if there's penetration (negative normal component)
                if c.z < -self.contact_slop {
                    let penetration = -c.z - self.contact_slop;
                    let correction = penetration * self.position_correction_rate;

                    let indices = constraint.body_indices();
                    if indices.len() == 2 {
                        let (idx_a, idx_b) = (indices[0], indices[1]);

                        // Handle static bodies
                        let (mass_a, mass_b) = if idx_a == usize::MAX {
                            (0.0, bodies[idx_b].mass)
                        } else if idx_b == usize::MAX {
                            (bodies[idx_a].mass, 0.0)
                        } else {
                            (bodies[idx_a].mass, bodies[idx_b].mass)
                        };

                        let inv_mass_a = if mass_a > 0.0 { 1.0 / mass_a } else { 0.0 };
                        let inv_mass_b = if mass_b > 0.0 { 1.0 / mass_b } else { 0.0 };

                        let total_inv_mass = inv_mass_a + inv_mass_b;
                        if total_inv_mass > 0.0 {
                            // Get contact normal from the constraint
                            if let Some((normal, _)) = constraint.get_contact_info() {
                                // Apply corrections
                                if idx_a != usize::MAX && inv_mass_a > 0.0 {
                                    let move_ratio = inv_mass_a / total_inv_mass;
                                    bodies[idx_a].position += normal * correction * move_ratio;
                                }
                                if idx_b != usize::MAX && inv_mass_b > 0.0 {
                                    let move_ratio = inv_mass_b / total_inv_mass;
                                    bodies[idx_b].position -= normal * correction * move_ratio;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_coloring() {
        let adjacency = vec![
            vec![1, 2],    // 0 connected to 1, 2
            vec![0, 2],    // 1 connected to 0, 2
            vec![0, 1, 3], // 2 connected to 0, 1, 3
            vec![2],       // 3 connected to 2
        ];

        let coloring = VertexColoring::greedy(&adjacency);

        // Check that adjacent vertices have different colors
        for (i, neighbors) in adjacency.iter().enumerate() {
            for &j in neighbors {
                assert_ne!(coloring.colors[i], coloring.colors[j]);
            }
        }
    }

    #[test]
    fn test_mat6_operations() {
        let a = Mat3::IDENTITY;
        let mat = Mat6::from_blocks(a, Mat3::ZERO, Mat3::ZERO, a);

        let (v1, v2) = mat.mul_vec6(Vec3::ONE, Vec3::new(2.0, 3.0, 4.0));
        assert_eq!(v1, Vec3::ONE);
        assert_eq!(v2, Vec3::new(2.0, 3.0, 4.0));
    }
}
