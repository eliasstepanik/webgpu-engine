//! Galaxy-scale coordinate system with hierarchical sectors
//!
//! Provides hierarchical coordinate structures for managing positions at galaxy scale
//! (10^20+ meters) where even f64 precision becomes insufficient for direct coordinates.

use glam::{DVec3, IVec3};
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Galaxy sector for hierarchical organization
///
/// Divides the galaxy into sectors to maintain precision at extreme scales.
/// Each sector represents a cube of space, typically 10^18 meters per side.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct GalaxySector {
    /// Sector coordinates in galaxy grid (integer coordinates)
    pub sector_coords: IVec3,
    /// Sector size in meters (typically 10^18 meters)
    pub sector_size: f64,
}

impl GalaxySector {
    /// Create a new galaxy sector
    pub fn new(sector_coords: IVec3, sector_size: f64) -> Self {
        Self {
            sector_coords,
            sector_size,
        }
    }

    /// Default sector size for galaxy-scale coordinates (10^18 meters)
    ///
    /// This allows for approximately 1000x1000x1000 sectors to cover
    /// a galaxy of 10^21 meters diameter.
    pub const DEFAULT_SECTOR_SIZE: f64 = 1e18;

    /// Get the world-space offset for this sector
    pub fn world_offset(&self) -> DVec3 {
        DVec3::new(
            self.sector_coords.x as f64 * self.sector_size,
            self.sector_coords.y as f64 * self.sector_size,
            self.sector_coords.z as f64 * self.sector_size,
        )
    }

    /// Check if this sector is adjacent to another sector
    pub fn is_adjacent_to(&self, other: &GalaxySector) -> bool {
        let diff = self.sector_coords - other.sector_coords;
        diff.x.abs() <= 1 && diff.y.abs() <= 1 && diff.z.abs() <= 1
    }

    /// Get the distance between sector centers
    pub fn distance_to(&self, other: &GalaxySector) -> f64 {
        let diff = self.sector_coords - other.sector_coords;
        let diff_vec = DVec3::new(diff.x as f64, diff.y as f64, diff.z as f64);
        diff_vec.length() * self.sector_size
    }
}

/// Hierarchical world position combining sector and local coordinates
///
/// This structure enables precise positioning at galaxy scales by combining
/// coarse sector positioning with fine local positioning within each sector.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct GalaxyPosition {
    /// Which sector this position is in
    pub sector: GalaxySector,
    /// Position within the sector (high precision)
    pub local_position: DVec3,
}

impl GalaxyPosition {
    /// Create a new galaxy position
    pub fn new(sector: GalaxySector, local_position: DVec3) -> Self {
        Self {
            sector,
            local_position,
        }
    }

    /// Convert to absolute world position
    ///
    /// Note: This may lose precision at extreme scales. Use for visualization
    /// or coarse calculations only.
    pub fn to_world_position(&self) -> DVec3 {
        self.sector.world_offset() + self.local_position
    }

    /// Create from absolute world position
    pub fn from_world_position(world_pos: DVec3, sector_size: f64) -> Self {
        let sector_coords = IVec3::new(
            (world_pos.x / sector_size).floor() as i32,
            (world_pos.y / sector_size).floor() as i32,
            (world_pos.z / sector_size).floor() as i32,
        );

        let sector_offset = DVec3::new(
            sector_coords.x as f64 * sector_size,
            sector_coords.y as f64 * sector_size,
            sector_coords.z as f64 * sector_size,
        );

        Self {
            sector: GalaxySector::new(sector_coords, sector_size),
            local_position: world_pos - sector_offset,
        }
    }

    /// Create from world position using default galaxy sector size
    pub fn from_world_position_default(world_pos: DVec3) -> Self {
        Self::from_world_position(world_pos, GalaxySector::DEFAULT_SECTOR_SIZE)
    }

    /// Get distance to another galaxy position
    ///
    /// Handles positions in different sectors correctly.
    pub fn distance_to(&self, other: &GalaxyPosition) -> f64 {
        if self.sector.sector_coords == other.sector.sector_coords {
            // Same sector - simple distance
            self.local_position.distance(other.local_position)
        } else {
            // Different sectors - use world positions
            // Note: This may lose some precision at extreme scales
            let world_self = self.to_world_position();
            let world_other = other.to_world_position();
            world_self.distance(world_other)
        }
    }

    /// Check if position is in the same sector as another
    pub fn same_sector_as(&self, other: &GalaxyPosition) -> bool {
        self.sector.sector_coords == other.sector.sector_coords
    }

    /// Normalize local position to stay within sector bounds
    ///
    /// If the local position exceeds sector boundaries, adjusts the sector
    /// coordinates and local position accordingly.
    pub fn normalize(&mut self) {
        let half_size = self.sector.sector_size * 0.5;

        // Check each axis
        for axis in 0..3 {
            let local_coord = match axis {
                0 => &mut self.local_position.x,
                1 => &mut self.local_position.y,
                2 => &mut self.local_position.z,
                _ => unreachable!(),
            };

            let sector_coord = match axis {
                0 => &mut self.sector.sector_coords.x,
                1 => &mut self.sector.sector_coords.y,
                2 => &mut self.sector.sector_coords.z,
                _ => unreachable!(),
            };

            // Move to adjacent sector if local position exceeds bounds
            while *local_coord > half_size {
                *local_coord -= self.sector.sector_size;
                *sector_coord += 1;
            }

            while *local_coord < -half_size {
                *local_coord += self.sector.sector_size;
                *sector_coord -= 1;
            }
        }
    }

    /// Translate by a given offset, handling sector boundaries
    pub fn translate(&mut self, offset: DVec3) {
        self.local_position += offset;
        self.normalize();
    }

    /// Convert to camera-relative position for rendering
    pub fn to_camera_relative(&self, camera_pos: &GalaxyPosition) -> DVec3 {
        if self.same_sector_as(camera_pos) {
            // Same sector - simple relative position
            self.local_position - camera_pos.local_position
        } else {
            // Different sectors - compute sector offset
            let sector_diff = self.sector.sector_coords - camera_pos.sector.sector_coords;
            let sector_offset = DVec3::new(
                sector_diff.x as f64 * self.sector.sector_size,
                sector_diff.y as f64 * self.sector.sector_size,
                sector_diff.z as f64 * self.sector.sector_size,
            );
            sector_offset + self.local_position - camera_pos.local_position
        }
    }

    /// Check if position is at galaxy scale (outside typical planetary range)
    pub fn is_galaxy_scale(&self) -> bool {
        self.sector.sector_coords != IVec3::ZERO || self.local_position.length() > 1e15
        // 1 million billion meters
    }

    /// Get a human-readable description of the position
    pub fn describe(&self) -> String {
        if self.is_galaxy_scale() {
            format!(
                "Galaxy position: sector {:?}, local offset {:.2} ly",
                self.sector.sector_coords,
                self.local_position.length() / 9.46e15
            )
        } else {
            format!(
                "Local position: {:.2} km from origin",
                self.local_position.length() / 1000.0
            )
        }
    }
}

/// Manager for galaxy-scale coordinate operations
pub struct GalaxyCoordinateSystem {
    /// Current camera galaxy position
    pub camera_position: GalaxyPosition,
    /// Maximum render distance in meters
    pub render_distance: f64,
    /// Sector size for this galaxy
    pub sector_size: f64,
}

impl GalaxyCoordinateSystem {
    /// Create a new galaxy coordinate system
    pub fn new(sector_size: f64) -> Self {
        Self {
            camera_position: GalaxyPosition::new(
                GalaxySector::new(IVec3::ZERO, sector_size),
                DVec3::ZERO,
            ),
            render_distance: 1e15, // Default 1 million billion meters
            sector_size,
        }
    }

    /// Create with default galaxy scale settings
    pub fn default_galaxy() -> Self {
        Self::new(GalaxySector::DEFAULT_SECTOR_SIZE)
    }

    /// Update camera position
    pub fn set_camera_position(&mut self, position: GalaxyPosition) {
        self.camera_position = position;
        debug!("Camera moved to {}", position.describe());
    }

    /// Check if a position should be rendered based on distance
    pub fn should_render(&self, position: &GalaxyPosition) -> bool {
        // Quick check for same sector
        if position.same_sector_as(&self.camera_position) {
            position
                .local_position
                .distance(self.camera_position.local_position)
                <= self.render_distance
        } else {
            // Check sector distance first for early rejection
            let sector_distance = position.sector.distance_to(&self.camera_position.sector);
            if sector_distance > self.render_distance * 2.0 {
                false
            } else {
                position.distance_to(&self.camera_position) <= self.render_distance
            }
        }
    }

    /// Get visible sectors based on render distance
    pub fn get_visible_sectors(&self) -> Vec<GalaxySector> {
        let mut visible_sectors = Vec::new();
        let sectors_to_check = (self.render_distance / self.sector_size).ceil() as i32 + 1;

        let camera_sector = self.camera_position.sector.sector_coords;

        for dx in -sectors_to_check..=sectors_to_check {
            for dy in -sectors_to_check..=sectors_to_check {
                for dz in -sectors_to_check..=sectors_to_check {
                    let sector_coords = camera_sector + IVec3::new(dx, dy, dz);
                    let sector = GalaxySector::new(sector_coords, self.sector_size);

                    // Check if any part of this sector is within render distance
                    let closest_point_in_sector = DVec3::new(
                        if dx > 0 {
                            -self.sector_size * 0.5
                        } else {
                            self.sector_size * 0.5
                        },
                        if dy > 0 {
                            -self.sector_size * 0.5
                        } else {
                            self.sector_size * 0.5
                        },
                        if dz > 0 {
                            -self.sector_size * 0.5
                        } else {
                            self.sector_size * 0.5
                        },
                    );

                    let test_pos = GalaxyPosition::new(sector, closest_point_in_sector);
                    if self.should_render(&test_pos) {
                        visible_sectors.push(sector);
                    }
                }
            }
        }

        visible_sectors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_galaxy_sector_creation() {
        let sector = GalaxySector::new(IVec3::new(1, -2, 3), 1e18);
        assert_eq!(sector.sector_coords, IVec3::new(1, -2, 3));
        assert_eq!(sector.sector_size, 1e18);

        let offset = sector.world_offset();
        assert_eq!(offset, DVec3::new(1e18, -2e18, 3e18));
    }

    #[test]
    fn test_galaxy_position_from_world() {
        let world_pos = DVec3::new(2.5e18, -1.3e18, 0.7e18);
        let galaxy_pos = GalaxyPosition::from_world_position(world_pos, 1e18);

        assert_eq!(galaxy_pos.sector.sector_coords, IVec3::new(2, -2, 0));

        let expected_local = DVec3::new(0.5e18, 0.7e18, 0.7e18);
        assert!((galaxy_pos.local_position - expected_local).length() < 1.0);

        // Verify reconstruction
        let reconstructed = galaxy_pos.to_world_position();
        assert!((reconstructed - world_pos).length() < 1.0);
    }

    #[test]
    fn test_galaxy_position_normalization() {
        let mut pos = GalaxyPosition::new(
            GalaxySector::new(IVec3::ZERO, 1e18),
            DVec3::new(0.6e18, -0.7e18, 0.3e18),
        );

        pos.normalize();

        // X should move to sector 1
        assert_eq!(pos.sector.sector_coords.x, 1);
        assert!((pos.local_position.x - (-0.4e18)).abs() < 1000.0);

        // Y should move to sector -1
        assert_eq!(pos.sector.sector_coords.y, -1);
        assert!((pos.local_position.y - 0.3e18).abs() < 1000.0);

        // Z should stay in sector 0
        assert_eq!(pos.sector.sector_coords.z, 0);
        assert!((pos.local_position.z - 0.3e18).abs() < 1000.0);
    }

    #[test]
    fn test_galaxy_position_distance() {
        let pos1 = GalaxyPosition::new(
            GalaxySector::new(IVec3::ZERO, 1e18),
            DVec3::new(0.0, 0.0, 0.0),
        );

        let pos2 = GalaxyPosition::new(
            GalaxySector::new(IVec3::ZERO, 1e18),
            DVec3::new(3e17, 4e17, 0.0),
        );

        // Same sector distance
        let dist = pos1.distance_to(&pos2);
        assert!((dist - 5e17).abs() < 1000.0);

        // Different sector distance
        let pos3 = GalaxyPosition::new(
            GalaxySector::new(IVec3::new(1, 0, 0), 1e18),
            DVec3::new(0.0, 0.0, 0.0),
        );

        let dist2 = pos1.distance_to(&pos3);
        assert!((dist2 - 1e18).abs() < 1e10); // Allow some precision loss
    }

    #[test]
    fn test_camera_relative_position() {
        let camera = GalaxyPosition::new(
            GalaxySector::new(IVec3::new(5, 5, 5), 1e18),
            DVec3::new(1e17, 2e17, 3e17),
        );

        let object = GalaxyPosition::new(
            GalaxySector::new(IVec3::new(5, 5, 5), 1e18),
            DVec3::new(2e17, 2e17, 3e17),
        );

        // Same sector
        let relative = object.to_camera_relative(&camera);
        assert!((relative.x - 1e17).abs() < 1000.0);
        assert!(relative.y.abs() < 1000.0);
        assert!(relative.z.abs() < 1000.0);

        // Different sector
        let distant_object = GalaxyPosition::new(
            GalaxySector::new(IVec3::new(6, 5, 5), 1e18),
            DVec3::new(0.0, 0.0, 0.0),
        );

        let relative2 = distant_object.to_camera_relative(&camera);
        assert!((relative2.x - 9e17).abs() < 1e10); // 1e18 - 1e17
    }

    #[test]
    fn test_galaxy_scale_precision() {
        // Test at Milky Way scale (10^20 meters)
        let galaxy_radius = 9.46e20; // ~100,000 light years
        let pos1 = GalaxyPosition::from_world_position_default(DVec3::new(galaxy_radius, 0.0, 0.0));
        let pos2 = GalaxyPosition::from_world_position_default(DVec3::new(
            galaxy_radius - 1000.0,
            0.0,
            0.0,
        ));

        let relative = pos1.to_camera_relative(&pos2);

        // At galaxy scale (10^20), f64 precision is ~10^5 meters
        // So we can't expect meter precision, but should be within 100km
        assert!((relative.x - 1000.0).abs() < 1e5);
    }

    #[test]
    fn test_coordinate_system_visible_sectors() {
        let mut coord_system = GalaxyCoordinateSystem::new(1e18);
        coord_system.render_distance = 2e18; // 2 sectors

        let visible = coord_system.get_visible_sectors();

        // Should include center and adjacent sectors
        assert!(visible.len() >= 7); // At least center + 6 face-adjacent
        assert!(visible.iter().any(|s| s.sector_coords == IVec3::ZERO));
    }
}
