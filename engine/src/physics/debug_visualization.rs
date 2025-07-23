//! Debug visualization for physics colliders

use crate::core::entity::World;
use crate::graphics::renderer::Renderer;
use crate::physics::components::{Collider, CollisionShape, Rigidbody};
use glam::{Mat4, Vec3, Vec4};
use tracing::trace;

/// Debug visualization state
pub struct PhysicsDebugVisualization {
    /// Whether debug visualization is enabled
    pub enabled: bool,
    /// Color for static colliders
    pub static_color: Vec4,
    /// Color for dynamic colliders
    pub dynamic_color: Vec4,
    /// Color for kinematic colliders
    pub kinematic_color: Vec4,
}

impl Default for PhysicsDebugVisualization {
    fn default() -> Self {
        Self {
            enabled: false,
            static_color: Vec4::new(0.0, 1.0, 0.0, 0.8), // Green
            dynamic_color: Vec4::new(1.0, 0.0, 0.0, 0.8), // Red
            kinematic_color: Vec4::new(0.0, 0.0, 1.0, 0.8), // Blue
        }
    }
}

impl PhysicsDebugVisualization {
    /// Update debug lines in the renderer based on current colliders
    pub fn update_debug_lines(&self, world: &World, renderer: &mut Renderer) {
        if !self.enabled {
            renderer.update_debug_lines(&[]);
            return;
        }

        let mut lines = Vec::new();

        // Query all entities with colliders and transforms
        let mut query = world.query::<(
            &Collider,
            &crate::core::entity::components::GlobalTransform,
            Option<&Rigidbody>,
        )>();

        for (entity, (collider, transform, rigid_body)) in query.iter() {
            // Determine color based on rigid body type
            let color = if let Some(rb) = rigid_body {
                if rb.is_kinematic {
                    self.kinematic_color
                } else {
                    self.dynamic_color
                }
            } else {
                self.static_color // No rigid body means static collider
            };

            // Extract scale, rotation, and position from transform
            let (scale, rotation, position) = transform.matrix.to_scale_rotation_translation();

            // Create transform without scale for wireframe (we'll apply scale to the shape)
            let transform_no_scale = Mat4::from_rotation_translation(rotation, position);

            // Generate wireframe based on collider shape with scale applied
            match &collider.shape {
                CollisionShape::Box { half_extents } => {
                    let scaled_extents = *half_extents * scale;
                    add_box_wireframe(&mut lines, transform_no_scale, scaled_extents, color);
                }
                CollisionShape::Sphere { radius } => {
                    let scaled_radius = *radius * scale.max_element();
                    add_sphere_wireframe(&mut lines, transform_no_scale, scaled_radius, color);
                }
                CollisionShape::Capsule {
                    radius,
                    half_height,
                } => {
                    let scaled_radius = *radius * scale.x.max(scale.z);
                    let scaled_half_height = *half_height * scale.y;
                    add_capsule_wireframe(
                        &mut lines,
                        transform_no_scale,
                        scaled_radius,
                        scaled_half_height * 2.0,
                        color,
                    );
                }
            }

            trace!(entity = ?entity, shape = ?collider.shape, "Generated debug wireframe for collider");
        }

        // Update renderer with the generated lines
        renderer.update_debug_lines(&lines);
    }
}

/// Add box wireframe lines
fn add_box_wireframe(lines: &mut Vec<f32>, transform: Mat4, half_extents: Vec3, color: Vec4) {
    // Define the 8 corners of the box in local space
    let corners = [
        Vec3::new(-half_extents.x, -half_extents.y, -half_extents.z),
        Vec3::new(half_extents.x, -half_extents.y, -half_extents.z),
        Vec3::new(half_extents.x, half_extents.y, -half_extents.z),
        Vec3::new(-half_extents.x, half_extents.y, -half_extents.z),
        Vec3::new(-half_extents.x, -half_extents.y, half_extents.z),
        Vec3::new(half_extents.x, -half_extents.y, half_extents.z),
        Vec3::new(half_extents.x, half_extents.y, half_extents.z),
        Vec3::new(-half_extents.x, half_extents.y, half_extents.z),
    ];

    // Transform corners to world space
    let world_corners: Vec<Vec3> = corners
        .iter()
        .map(|&corner| transform.transform_point3(corner))
        .collect();

    // Define the 12 edges of the box
    let edges = [
        // Bottom face
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        // Top face
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        // Vertical edges
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];

    // Add lines for each edge
    for &(i, j) in &edges {
        add_line(lines, world_corners[i], world_corners[j], color);
    }
}

/// Add sphere wireframe lines (approximated with circles)
fn add_sphere_wireframe(lines: &mut Vec<f32>, transform: Mat4, radius: f32, color: Vec4) {
    const SEGMENTS: usize = 16;

    // Extract position and rotation from transform
    let (scale, rotation, position) = transform.to_scale_rotation_translation();
    let scaled_radius = radius * scale.max_element();

    // Generate three circles (XY, XZ, YZ planes)
    let planes = [
        (Vec3::X, Vec3::Y), // XY plane
        (Vec3::X, Vec3::Z), // XZ plane
        (Vec3::Y, Vec3::Z), // YZ plane
    ];

    for (axis1, axis2) in &planes {
        let mut prev_point = None;
        let mut first_point = Vec3::ZERO;

        // Generate circle points
        for i in 0..=SEGMENTS {
            let angle = (i as f32 / SEGMENTS as f32) * std::f32::consts::TAU;
            let local_point =
                *axis1 * angle.cos() * scaled_radius + *axis2 * angle.sin() * scaled_radius;
            let world_point = position + rotation * local_point;

            if i == 0 {
                first_point = world_point;
            }

            if let Some(prev) = prev_point {
                add_line(lines, prev, world_point, color);
            }

            prev_point = Some(world_point);

            // Close the circle
            if i == SEGMENTS {
                if let Some(prev) = prev_point {
                    add_line(lines, prev, first_point, color);
                }
            }
        }
    }
}

/// Add capsule wireframe lines
fn add_capsule_wireframe(
    lines: &mut Vec<f32>,
    transform: Mat4,
    radius: f32,
    height: f32,
    color: Vec4,
) {
    const SEGMENTS: usize = 16;

    // Extract transform components
    let (scale, rotation, position) = transform.to_scale_rotation_translation();
    let scaled_radius = radius * scale.x.max(scale.z); // Use XZ scale for radius
    let scaled_height = height * scale.y;

    // Half height of the cylindrical part
    let half_height = scaled_height * 0.5;

    // Generate circles at top and bottom
    for y_offset in &[-half_height, half_height] {
        let mut prev_point = None;
        let mut first_point = Vec3::ZERO;

        for i in 0..=SEGMENTS {
            let angle = (i as f32 / SEGMENTS as f32) * std::f32::consts::TAU;
            let local_point = Vec3::new(
                angle.cos() * scaled_radius,
                *y_offset,
                angle.sin() * scaled_radius,
            );
            let world_point = transform.transform_point3(local_point);

            if i == 0 {
                first_point = world_point;
            }

            if let Some(prev) = prev_point {
                add_line(lines, prev, world_point, color);
            }

            prev_point = Some(world_point);

            // Close the circle
            if i == SEGMENTS {
                if let Some(prev) = prev_point {
                    add_line(lines, prev, first_point, color);
                }
            }
        }
    }

    // Generate vertical lines connecting the circles
    for i in 0..4 {
        let angle = (i as f32 / 4.0) * std::f32::consts::TAU;
        let x = angle.cos() * scaled_radius;
        let z = angle.sin() * scaled_radius;

        let bottom_point = transform.transform_point3(Vec3::new(x, -half_height, z));
        let top_point = transform.transform_point3(Vec3::new(x, half_height, z));

        add_line(lines, bottom_point, top_point, color);
    }

    // Add hemisphere outlines (simplified as arcs)
    for i in 0..2 {
        let angle = (i as f32 / 2.0) * std::f32::consts::TAU;
        let x_dir = Vec3::new(angle.cos(), 0.0, angle.sin());

        // Top hemisphere
        add_hemisphere_arc(
            lines,
            position + rotation * Vec3::new(0.0, half_height, 0.0),
            rotation * x_dir,
            rotation * Vec3::Y,
            scaled_radius,
            color,
            true,
        );

        // Bottom hemisphere
        add_hemisphere_arc(
            lines,
            position + rotation * Vec3::new(0.0, -half_height, 0.0),
            rotation * x_dir,
            rotation * -Vec3::Y,
            scaled_radius,
            color,
            false,
        );
    }
}

/// Add a hemisphere arc for capsule ends
fn add_hemisphere_arc(
    lines: &mut Vec<f32>,
    center: Vec3,
    right: Vec3,
    up: Vec3,
    radius: f32,
    color: Vec4,
    is_top: bool,
) {
    const SEGMENTS: usize = 8;
    let mut prev_point = None;

    for i in 0..=SEGMENTS {
        let t = i as f32 / SEGMENTS as f32;
        let angle = if is_top {
            t * std::f32::consts::PI * 0.5
        } else {
            std::f32::consts::PI * 0.5 + t * std::f32::consts::PI * 0.5
        };

        let point = center + right * angle.sin() * radius + up * angle.cos() * radius;

        if let Some(prev) = prev_point {
            add_line(lines, prev, point, color);
        }

        prev_point = Some(point);
    }
}

/// Add a single line to the line buffer
fn add_line(lines: &mut Vec<f32>, start: Vec3, end: Vec3, color: Vec4) {
    // Start vertex
    lines.push(start.x);
    lines.push(start.y);
    lines.push(start.z);
    lines.push(color.x);
    lines.push(color.y);
    lines.push(color.z);
    lines.push(color.w);

    // End vertex
    lines.push(end.x);
    lines.push(end.y);
    lines.push(end.z);
    lines.push(color.x);
    lines.push(color.y);
    lines.push(color.z);
    lines.push(color.w);
}
