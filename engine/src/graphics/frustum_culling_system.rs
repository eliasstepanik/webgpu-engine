//! Frustum culling system for efficient rendering

use crate::core::entity::components::GlobalTransform;
use crate::core::entity::World;
use crate::graphics::{Frustum, MeshId, Renderer, Visibility, AABB};
use crate::profile_zone;
use glam::{DVec3, Mat4, Vec3};
use tracing::debug;

/// Perform frustum culling on entities with AABB and Visibility components
pub fn frustum_culling_system(
    world: &mut World,
    camera_view_proj: Mat4,
    camera_world_position: DVec3,
    renderer: &Renderer,
) {
    profile_zone!("frustum_culling_system");

    // Extract frustum from view-projection matrix
    let frustum = Frustum::from_matrix(camera_view_proj);

    let mut visible_count = 0;
    let mut culled_count = 0;

    // First pass: collect entities that need visibility updates
    let mut entities_to_update = Vec::new();

    for (entity, (mesh_id, global_transform)) in world.query::<(&MeshId, &GlobalTransform)>().iter()
    {
        // Get or calculate AABB for this entity
        let mesh_aabb = match renderer.get_mesh_aabb(mesh_id) {
            Some(aabb) => aabb,
            None => {
                // Skip entities without valid mesh AABBs
                continue;
            }
        };

        // Transform AABB to world space
        let world_aabb = mesh_aabb.transform(global_transform.matrix);

        // Transform AABB to camera-relative space for large world support
        let relative_aabb = transform_aabb_to_camera_space(&world_aabb, camera_world_position);

        // Test visibility
        let is_visible = frustum.is_aabb_visible(&relative_aabb);

        entities_to_update.push((entity, is_visible));

        if is_visible {
            visible_count += 1;
        } else {
            culled_count += 1;
        }
    }

    // Second pass: update visibility components
    for (entity, is_visible) in entities_to_update {
        // Get existing visibility or create new one
        let mut visibility = world
            .get::<Visibility>(entity)
            .map(|v| *v)
            .unwrap_or_else(|_| Visibility::new());

        // Update visibility state
        visibility.update(is_visible);

        // Insert/update the component
        let _ = world.insert_one(entity, visibility);
    }

    debug!(
        visible = visible_count,
        culled = culled_count,
        "Frustum culling complete"
    );
}

/// Transform AABB to camera-relative space for large world precision
fn transform_aabb_to_camera_space(world_aabb: &AABB, camera_world_position: DVec3) -> AABB {
    // Convert to camera-relative coordinates
    let camera_offset = Vec3::new(
        camera_world_position.x as f32,
        camera_world_position.y as f32,
        camera_world_position.z as f32,
    );

    AABB::new(
        world_aabb.min - camera_offset,
        world_aabb.max - camera_offset,
    )
}

/// Alternative system that works with entities that already have AABB components
pub fn frustum_culling_system_with_aabb_components(
    world: &mut World,
    camera_view_proj: Mat4,
    camera_world_position: DVec3,
) {
    profile_zone!("frustum_culling_system_with_aabb");

    let frustum = Frustum::from_matrix(camera_view_proj);

    let mut visible_count = 0;
    let mut culled_count = 0;

    // First pass: collect entities to update
    let mut entities_to_update = Vec::new();

    for (entity, (aabb, global_transform)) in world.query::<(&AABB, &GlobalTransform)>().iter() {
        // Transform AABB to world space
        let world_aabb = aabb.transform(global_transform.matrix);

        // Transform to camera-relative space
        let relative_aabb = transform_aabb_to_camera_space(&world_aabb, camera_world_position);

        // Test visibility
        let is_visible = frustum.is_aabb_visible(&relative_aabb);
        entities_to_update.push((entity, is_visible));

        if is_visible {
            visible_count += 1;
        } else {
            culled_count += 1;
        }
    }

    // Second pass: update visibility components
    for (entity, is_visible) in entities_to_update {
        // Get current visibility in a limited scope
        let mut visibility = {
            if let Ok(visibility_ref) = world.get::<Visibility>(entity) {
                *visibility_ref
            } else {
                continue;
            }
        };

        // Update and insert
        visibility.update(is_visible);
        let _ = world.insert_one(entity, visibility);
    }

    debug!(
        visible = visible_count,
        culled = culled_count,
        "Frustum culling with AABB components complete"
    );
}

/// Initialize AABB components for entities with meshes
pub fn initialize_mesh_aabbs(world: &mut World, renderer: &Renderer) {
    profile_zone!("initialize_mesh_aabbs");

    let mut initialized_count = 0;

    // Collect entities that need AABBs
    let mut entities_to_update = Vec::new();
    for (entity, mesh_id) in world.query::<&MeshId>().iter() {
        // Skip if entity already has AABB
        if world.get::<&AABB>(entity).is_ok() {
            continue;
        }

        // Get mesh AABB from renderer
        if let Some(mesh_aabb) = renderer.get_mesh_aabb(mesh_id) {
            entities_to_update.push((entity, mesh_aabb));
        }
    }

    // Now update entities without borrow conflicts
    for (entity, mesh_aabb) in entities_to_update {
        if world.insert_one(entity, mesh_aabb).is_ok() {
            initialized_count += 1;
        }
    }

    if initialized_count > 0 {
        debug!(
            count = initialized_count,
            "Initialized AABB components for entities"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn test_transform_aabb_to_camera_space() {
        let world_aabb = AABB::new(Vec3::new(100.0, 0.0, 100.0), Vec3::new(110.0, 10.0, 110.0));

        let camera_pos = DVec3::new(100.0, 5.0, 100.0);

        let relative_aabb = transform_aabb_to_camera_space(&world_aabb, camera_pos);

        // AABB should now be centered around origin in camera space
        assert_eq!(relative_aabb.min, Vec3::new(0.0, -5.0, 0.0));
        assert_eq!(relative_aabb.max, Vec3::new(10.0, 5.0, 10.0));
    }
}
