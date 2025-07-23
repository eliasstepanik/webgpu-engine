//! Broad phase collision detection using sweep and prune

use super::AABB;
use glam::Vec3;
use hecs::Entity;
use std::cmp::Ordering;

/// Entry for broad phase collision detection
pub struct BroadPhaseEntry {
    pub entity: Entity,
    pub aabb: AABB,
}

/// Axis for sweep and prune
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    X,
    Y,
    Z,
}

/// Endpoint for sweep and prune
struct Endpoint {
    value: f32,
    index: usize,
    is_min: bool,
}

impl Endpoint {
    fn new(value: f32, index: usize, is_min: bool) -> Self {
        Self {
            value,
            index,
            is_min,
        }
    }
}

/// Perform broad phase collision detection using sweep and prune
pub fn sweep_and_prune(entries: &[BroadPhaseEntry]) -> Vec<(usize, usize)> {
    if entries.len() < 2 {
        return Vec::new();
    }

    // Determine the best axis to sort along (highest variance)
    let axis = determine_best_axis(entries);

    // Create and sort endpoints
    let mut endpoints = create_endpoints(entries, axis);
    endpoints.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap_or(Ordering::Equal));

    // Sweep and find overlapping pairs
    let mut pairs = Vec::new();
    let mut active: Vec<usize> = Vec::new();

    for endpoint in endpoints {
        if endpoint.is_min {
            // Check against all active intervals
            for &active_index in &active {
                if active_index != endpoint.index {
                    let entry_a: &BroadPhaseEntry = &entries[active_index];
                    let entry_b: &BroadPhaseEntry = &entries[endpoint.index];
                    if entry_a.aabb.overlaps(&entry_b.aabb) {
                        let min_index = active_index.min(endpoint.index);
                        let max_index = active_index.max(endpoint.index);
                        pairs.push((min_index, max_index));
                    }
                }
            }
            active.push(endpoint.index);
        } else {
            // Remove from active list
            active.retain(|&idx| idx != endpoint.index);
        }
    }

    // Remove duplicates
    pairs.sort_unstable();
    pairs.dedup();

    pairs
}

/// Determine the best axis for sweep and prune based on variance
fn determine_best_axis(entries: &[BroadPhaseEntry]) -> Axis {
    let mut mean = Vec3::ZERO;
    let mut variance = Vec3::ZERO;

    // Calculate mean
    for entry in entries {
        mean += entry.aabb.center();
    }
    mean /= entries.len() as f32;

    // Calculate variance
    for entry in entries {
        let diff = entry.aabb.center() - mean;
        variance += diff * diff;
    }
    variance /= entries.len() as f32;

    // Choose axis with highest variance
    if variance.x > variance.y && variance.x > variance.z {
        Axis::X
    } else if variance.y > variance.z {
        Axis::Y
    } else {
        Axis::Z
    }
}

/// Create endpoints for sweep and prune
fn create_endpoints(entries: &[BroadPhaseEntry], axis: Axis) -> Vec<Endpoint> {
    let mut endpoints = Vec::with_capacity(entries.len() * 2);

    for (index, entry) in entries.iter().enumerate() {
        let (min_val, max_val) = match axis {
            Axis::X => (entry.aabb.min.x, entry.aabb.max.x),
            Axis::Y => (entry.aabb.min.y, entry.aabb.max.y),
            Axis::Z => (entry.aabb.min.z, entry.aabb.max.z),
        };

        endpoints.push(Endpoint::new(min_val, index, true));
        endpoints.push(Endpoint::new(max_val, index, false));
    }

    endpoints
}

/// Spatial hash for broad phase collision detection (alternative to sweep and prune)
pub struct SpatialHash {
    cell_size: f32,
    buckets: std::collections::HashMap<(i32, i32, i32), Vec<usize>>,
}

impl SpatialHash {
    /// Create a new spatial hash with the given cell size
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            buckets: std::collections::HashMap::new(),
        }
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.buckets.clear();
    }

    /// Insert an entry into the spatial hash
    pub fn insert(&mut self, index: usize, aabb: &AABB) {
        let min_cell = self.world_to_cell(aabb.min);
        let max_cell = self.world_to_cell(aabb.max);

        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                for z in min_cell.2..=max_cell.2 {
                    self.buckets.entry((x, y, z)).or_default().push(index);
                }
            }
        }
    }

    /// Query for potential collisions with an AABB
    pub fn query(&self, aabb: &AABB) -> Vec<usize> {
        let min_cell = self.world_to_cell(aabb.min);
        let max_cell = self.world_to_cell(aabb.max);

        let mut results = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                for z in min_cell.2..=max_cell.2 {
                    if let Some(indices) = self.buckets.get(&(x, y, z)) {
                        for &index in indices {
                            if seen.insert(index) {
                                results.push(index);
                            }
                        }
                    }
                }
            }
        }

        results
    }

    /// Convert world position to cell coordinates
    fn world_to_cell(&self, pos: Vec3) -> (i32, i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
            (pos.z / self.cell_size).floor() as i32,
        )
    }
}

/// Simple O(n²) broad phase for small numbers of objects
pub fn brute_force_pairs(entries: &[BroadPhaseEntry]) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();

    for i in 0..entries.len() {
        for j in (i + 1)..entries.len() {
            if entries[i].aabb.overlaps(&entries[j].aabb) {
                pairs.push((i, j));
            }
        }
    }

    pairs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sweep_and_prune() {
        // Create dummy entities using hecs::World
        let mut world = hecs::World::new();
        let entity0 = world.spawn(());
        let entity1 = world.spawn(());
        let entity2 = world.spawn(());

        let entries = vec![
            BroadPhaseEntry {
                entity: entity0,
                aabb: AABB::new(Vec3::ZERO, Vec3::ONE),
            },
            BroadPhaseEntry {
                entity: entity1,
                aabb: AABB::new(Vec3::splat(0.5), Vec3::splat(1.5)),
            },
            BroadPhaseEntry {
                entity: entity2,
                aabb: AABB::new(Vec3::splat(10.0), Vec3::splat(11.0)),
            },
        ];

        let pairs = sweep_and_prune(&entries);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], (0, 1));
    }

    #[test]
    fn test_spatial_hash() {
        let mut hash = SpatialHash::new(1.0);

        let aabb1 = AABB::new(Vec3::ZERO, Vec3::ONE);
        let aabb2 = AABB::new(Vec3::splat(0.5), Vec3::splat(1.5));
        let aabb3 = AABB::new(Vec3::splat(10.0), Vec3::splat(11.0));

        hash.insert(0, &aabb1);
        hash.insert(1, &aabb2);
        hash.insert(2, &aabb3);

        let query = hash.query(&aabb1);
        assert!(query.contains(&0));
        assert!(query.contains(&1));
        assert!(!query.contains(&2));
    }
}
