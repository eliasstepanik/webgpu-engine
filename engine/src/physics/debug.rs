//! Physics debug visualization
//!
//! This module provides debug rendering for physics colliders and constraints,
//! converting physics shapes to debug lines in camera-relative space.

use crate::core::entity::{World, WorldTransform, Transform};
use crate::dev::debug_overlay::DebugLineData;
use crate::physics::{Collider, ColliderShape, PhysicsWorld, RigidBody};
use glam::{DVec3, Vec3, Vec4};
use tracing::trace;

/// Settings for physics debug visualization
#[derive(Debug, Clone)]
pub struct PhysicsDebugSettings {
    /// Whether to show collider shapes
    pub show_colliders: bool,
    /// Color for static colliders
    pub static_color: Vec4,
    /// Color for dynamic colliders
    pub dynamic_color: Vec4,
    /// Color for kinematic colliders
    pub kinematic_color: Vec4,
    /// Color for sensor colliders
    pub sensor_color: Vec4,
}

impl Default for PhysicsDebugSettings {
    fn default() -> Self {
        Self {
            show_colliders: false,
            static_color: Vec4::new(0.0, 1.0, 0.0, 1.0),    // Green
            dynamic_color: Vec4::new(1.0, 0.0, 0.0, 1.0),   // Red
            kinematic_color: Vec4::new(0.0, 0.0, 1.0, 1.0), // Blue
            sensor_color: Vec4::new(1.0, 1.0, 0.0, 0.5),    // Yellow (semi-transparent)
        }
    }
}

/// Draw physics debug visualization
pub fn draw_physics_debug(
    world: &World,
    physics_world: &PhysicsWorld,
    debug_lines: &mut Vec<DebugLineData>,
    settings: &PhysicsDebugSettings,
    camera_world_position: DVec3,
) {
    if !settings.show_colliders {
        return;
    }
    
    trace!("Drawing physics debug visualization");
    
    // Draw colliders
    for (entity, (collider, rb)) in world.query::<(&Collider, &RigidBody)>().iter() {
        if let Some(rb_handle) = rb.handle {
            if let Some(rigid_body) = physics_world.rigid_body_set.get(rb_handle) {
                let pos = rigid_body.translation();
                let rot = rigid_body.rotation();
                
                // Determine color based on body type and sensor status
                let color = if collider.is_sensor {
                    settings.sensor_color
                } else {
                    match rb.body_type {
                        crate::physics::RigidBodyType::Fixed => settings.static_color,
                        crate::physics::RigidBodyType::Dynamic => settings.dynamic_color,
                        _ => settings.kinematic_color,
                    }
                };
                
                // Get world position (high precision)
                let world_pos = DVec3::new(pos.x, pos.y, pos.z);
                
                // Convert to camera-relative position
                let relative_pos = world_pos - camera_world_position;
                let relative_pos_f32 = Vec3::new(
                    relative_pos.x as f32,
                    relative_pos.y as f32,
                    relative_pos.z as f32,
                );
                
                // Draw the collider shape
                match &collider.shape {
                    ColliderShape::Cuboid(half_extents) => {
                        draw_box(
                            debug_lines,
                            relative_pos_f32,
                            rot.into(),
                            *half_extents,
                            color,
                        );
                    }
                    ColliderShape::Sphere(radius) => {
                        draw_sphere(
                            debug_lines,
                            relative_pos_f32,
                            *radius,
                            color,
                        );
                    }
                    ColliderShape::Capsule { half_height, radius } => {
                        draw_capsule(
                            debug_lines,
                            relative_pos_f32,
                            rot.into(),
                            *half_height,
                            *radius,
                            color,
                        );
                    }
                    ColliderShape::Cylinder { half_height, radius } => {
                        draw_cylinder(
                            debug_lines,
                            relative_pos_f32,
                            rot.into(),
                            *half_height,
                            *radius,
                            color,
                        );
                    }
                }
            }
        }
    }
}

/// Draw a box wireframe
fn draw_box(
    debug_lines: &mut Vec<DebugLineData>,
    position: Vec3,
    rotation: glam::Quat,
    half_extents: Vec3,
    color: Vec4,
) {
    let corners = [
        Vec3::new(-half_extents.x, -half_extents.y, -half_extents.z),
        Vec3::new( half_extents.x, -half_extents.y, -half_extents.z),
        Vec3::new( half_extents.x,  half_extents.y, -half_extents.z),
        Vec3::new(-half_extents.x,  half_extents.y, -half_extents.z),
        Vec3::new(-half_extents.x, -half_extents.y,  half_extents.z),
        Vec3::new( half_extents.x, -half_extents.y,  half_extents.z),
        Vec3::new( half_extents.x,  half_extents.y,  half_extents.z),
        Vec3::new(-half_extents.x,  half_extents.y,  half_extents.z),
    ];
    
    // Transform corners to world space
    let transformed_corners: Vec<Vec3> = corners
        .iter()
        .map(|&corner| position + rotation * corner)
        .collect();
    
    // Draw edges
    let edges = [
        (0, 1), (1, 2), (2, 3), (3, 0), // Bottom face
        (4, 5), (5, 6), (6, 7), (7, 4), // Top face
        (0, 4), (1, 5), (2, 6), (3, 7), // Vertical edges
    ];
    
    for &(i, j) in &edges {
        debug_lines.push(DebugLineData {
            start: transformed_corners[i],
            end: transformed_corners[j],
            color,
        });
    }
}

/// Draw a sphere wireframe
fn draw_sphere(
    debug_lines: &mut Vec<DebugLineData>,
    position: Vec3,
    radius: f32,
    color: Vec4,
) {
    const SEGMENTS: usize = 16;
    
    // Draw three circles for the sphere
    for axis in 0..3 {
        for i in 0..SEGMENTS {
            let angle1 = (i as f32) * 2.0 * std::f32::consts::PI / SEGMENTS as f32;
            let angle2 = ((i + 1) % SEGMENTS) as f32 * 2.0 * std::f32::consts::PI / SEGMENTS as f32;
            
            let (sin1, cos1) = angle1.sin_cos();
            let (sin2, cos2) = angle2.sin_cos();
            
            let p1 = match axis {
                0 => Vec3::new(0.0, sin1 * radius, cos1 * radius),
                1 => Vec3::new(sin1 * radius, 0.0, cos1 * radius),
                _ => Vec3::new(sin1 * radius, cos1 * radius, 0.0),
            };
            
            let p2 = match axis {
                0 => Vec3::new(0.0, sin2 * radius, cos2 * radius),
                1 => Vec3::new(sin2 * radius, 0.0, cos2 * radius),
                _ => Vec3::new(sin2 * radius, cos2 * radius, 0.0),
            };
            
            debug_lines.push(DebugLineData {
                start: position + p1,
                end: position + p2,
                color,
            });
        }
    }
}

/// Draw a capsule wireframe (Y-axis aligned)
fn draw_capsule(
    debug_lines: &mut Vec<DebugLineData>,
    position: Vec3,
    rotation: glam::Quat,
    half_height: f32,
    radius: f32,
    color: Vec4,
) {
    const SEGMENTS: usize = 16;
    
    // Draw cylinder part
    for i in 0..SEGMENTS {
        let angle = (i as f32) * 2.0 * std::f32::consts::PI / SEGMENTS as f32;
        let next_angle = ((i + 1) % SEGMENTS) as f32 * 2.0 * std::f32::consts::PI / SEGMENTS as f32;
        
        let (sin1, cos1) = angle.sin_cos();
        let (sin2, cos2) = next_angle.sin_cos();
        
        // Top circle
        let top1 = rotation * Vec3::new(sin1 * radius, half_height, cos1 * radius);
        let top2 = rotation * Vec3::new(sin2 * radius, half_height, cos2 * radius);
        
        // Bottom circle
        let bot1 = rotation * Vec3::new(sin1 * radius, -half_height, cos1 * radius);
        let bot2 = rotation * Vec3::new(sin2 * radius, -half_height, cos2 * radius);
        
        // Horizontal lines
        debug_lines.push(DebugLineData {
            start: position + top1,
            end: position + top2,
            color,
        });
        
        debug_lines.push(DebugLineData {
            start: position + bot1,
            end: position + bot2,
            color,
        });
        
        // Vertical lines (every 4th segment)
        if i % 4 == 0 {
            debug_lines.push(DebugLineData {
                start: position + top1,
                end: position + bot1,
                color,
            });
        }
    }
    
    // Draw hemisphere caps (simplified)
    for i in 0..SEGMENTS / 2 {
        let angle = (i as f32) * std::f32::consts::PI / (SEGMENTS / 2) as f32;
        let next_angle = ((i + 1) % (SEGMENTS / 2)) as f32 * std::f32::consts::PI / (SEGMENTS / 2) as f32;
        
        let y1 = angle.cos() * radius;
        let r1 = angle.sin() * radius;
        let y2 = next_angle.cos() * radius;
        let r2 = next_angle.sin() * radius;
        
        // Draw meridians
        for j in 0..4 {
            let phi = (j as f32) * 2.0 * std::f32::consts::PI / 4.0;
            let (sin_phi, cos_phi) = phi.sin_cos();
            
            let p1_top = rotation * Vec3::new(sin_phi * r1, half_height + y1, cos_phi * r1);
            let p2_top = rotation * Vec3::new(sin_phi * r2, half_height + y2, cos_phi * r2);
            
            let p1_bot = rotation * Vec3::new(sin_phi * r1, -half_height - y1, cos_phi * r1);
            let p2_bot = rotation * Vec3::new(sin_phi * r2, -half_height - y2, cos_phi * r2);
            
            debug_lines.push(DebugLineData {
                start: position + p1_top,
                end: position + p2_top,
                color,
            });
            
            debug_lines.push(DebugLineData {
                start: position + p1_bot,
                end: position + p2_bot,
                color,
            });
        }
    }
}

/// Draw a cylinder wireframe (Y-axis aligned)
fn draw_cylinder(
    debug_lines: &mut Vec<DebugLineData>,
    position: Vec3,
    rotation: glam::Quat,
    half_height: f32,
    radius: f32,
    color: Vec4,
) {
    const SEGMENTS: usize = 16;
    
    for i in 0..SEGMENTS {
        let angle = (i as f32) * 2.0 * std::f32::consts::PI / SEGMENTS as f32;
        let next_angle = ((i + 1) % SEGMENTS) as f32 * 2.0 * std::f32::consts::PI / SEGMENTS as f32;
        
        let (sin1, cos1) = angle.sin_cos();
        let (sin2, cos2) = next_angle.sin_cos();
        
        // Top circle
        let top1 = rotation * Vec3::new(sin1 * radius, half_height, cos1 * radius);
        let top2 = rotation * Vec3::new(sin2 * radius, half_height, cos2 * radius);
        
        // Bottom circle
        let bot1 = rotation * Vec3::new(sin1 * radius, -half_height, cos1 * radius);
        let bot2 = rotation * Vec3::new(sin2 * radius, -half_height, cos2 * radius);
        
        // Horizontal lines
        debug_lines.push(DebugLineData {
            start: position + top1,
            end: position + top2,
            color,
        });
        
        debug_lines.push(DebugLineData {
            start: position + bot1,
            end: position + bot2,
            color,
        });
        
        // Vertical lines (every 4th segment)
        if i % 4 == 0 {
            debug_lines.push(DebugLineData {
                start: position + top1,
                end: position + bot1,
                color,
            });
        }
    }
}