name: "Milky Way Scale Coordinate Support"
description: |

## Purpose
Extend the existing large world coordinate system to support Milky Way galaxy scale (100,000+ light-years / ~10^21 meters) by implementing hierarchical coordinate systems, enhanced logarithmic depth buffer support, and galaxy-scale validation tests.

## Core Principles
1. **Hierarchical Coordinates**: Implement sector/region system for galaxy-scale organization
2. **Backward Compatible**: Preserve existing planetary-scale functionality
3. **Precision Aware**: Validate f64 precision limits at extreme scales
4. **Performance Optimized**: Minimize overhead for extreme coordinate transformations
5. **Logarithmic Depth**: Enhance depth buffer for extreme near/far ratios

---

## Goal
Enable the WebGPU engine to handle Milky Way galaxy scale worlds (~10^21 meters) while maintaining precision and performance through hierarchical coordinate systems and enhanced logarithmic depth rendering.

## Why
- **Scale Requirements**: Current system targets >1 million units, galaxy scale needs ~10^21 meters
- **Scientific Accuracy**: 1 light-year = 9.46×10^15 meters, Milky Way = ~9.46×10^20 meters diameter
- **Float64 Capability**: Maximum value ~1.8×10^308 theoretically supports galaxy scale
- **Game Types**: Enable space exploration, galaxy simulations, cosmic-scale games
- **Hierarchical Organization**: Galaxy/sector/system/planet hierarchy mirrors real astronomy

## What
Implement a hierarchical coordinate system with galaxy sectors, enhance logarithmic depth buffer support, and add galaxy-scale validation tests while maintaining backward compatibility.

### Success Criteria
- [ ] Objects remain stable at 10^21 meters from origin (galaxy scale)
- [ ] Hierarchical coordinate system with sectors/regions implemented
- [ ] Logarithmic depth buffer handles 10^-1 to 10^21 meter ranges
- [ ] Performance remains acceptable with galaxy-scale transformations
- [ ] All existing tests pass (backward compatibility)
- [ ] New galaxy-scale scene examples render correctly

---

## Implementation Blueprint

### 1. Hierarchical Coordinate System

```rust
// engine/src/core/coordinates/galaxy_coordinates.rs

/// Galaxy sector for hierarchical organization
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GalaxySector {
    /// Sector coordinates in galaxy grid (integer coordinates)
    pub sector_coords: IVec3,
    /// Sector size in meters (typically 10^18 meters)
    pub sector_size: f64,
}

/// Hierarchical world position combining sector and local coordinates
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GalaxyPosition {
    /// Which sector this position is in
    pub sector: GalaxySector,
    /// Position within the sector (high precision)
    pub local_position: DVec3,
}

impl GalaxyPosition {
    /// Convert to absolute world position
    pub fn to_world_position(&self) -> DVec3 {
        let sector_offset = DVec3::new(
            self.sector.sector_coords.x as f64 * self.sector.sector_size,
            self.sector.sector_coords.y as f64 * self.sector.sector_size,
            self.sector.sector_coords.z as f64 * self.sector.sector_size,
        );
        sector_offset + self.local_position
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
            sector: GalaxySector { sector_coords, sector_size },
            local_position: world_pos - sector_offset,
        }
    }
}
```

### 2. Enhanced WorldTransform Component

```rust
// Extend engine/src/core/coordinates/world_transform.rs

impl WorldTransform {
    /// Convert to hierarchical galaxy position
    pub fn to_galaxy_position(&self, sector_size: f64) -> GalaxyPosition {
        GalaxyPosition::from_world_position(self.position, sector_size)
    }
    
    /// Create from hierarchical galaxy position
    pub fn from_galaxy_position(galaxy_pos: &GalaxyPosition) -> Self {
        Self::from_position(galaxy_pos.to_world_position())
    }
    
    /// Check if position is at galaxy scale (>10^15 meters from origin)
    pub fn is_galaxy_scale(&self) -> bool {
        self.position.length() > 1e15
    }
}
```

### 3. Enhanced Logarithmic Depth

```rust
// Update engine/src/core/camera.rs

impl Camera {
    /// Create a camera optimized for galaxy scale rendering
    pub fn galaxy_scale(fov_y_degrees: f32, aspect_ratio: f32) -> Self {
        Self {
            fov_y_radians: fov_y_degrees.to_radians(),
            aspect_ratio,
            z_near: 0.1,  // 10cm minimum
            z_far: 1e21,  // Galaxy scale maximum
            projection_mode: ProjectionMode::Perspective,
            use_logarithmic_depth: true,  // Always use logarithmic for galaxy scale
        }
    }
    
    /// Get enhanced logarithmic depth coefficient for extreme ranges
    pub fn galaxy_logarithmic_depth_coefficient(&self) -> f32 {
        // Enhanced formula for extreme near/far ratios
        let fc = 2.0 / (self.z_far / self.z_near).ln();
        fc * (1.0 + self.z_near / self.z_far)
    }
}
```

### 4. Galaxy Scale Tests

```rust
// Add to engine/src/core/coordinates/tests.rs

#[test]
fn test_galaxy_scale_precision() {
    // Test at Milky Way scale (10^20 meters)
    let galaxy_radius = 9.46e20; // ~100,000 light years
    let world_pos = DVec3::new(galaxy_radius, 0.0, 0.0);
    let camera_pos = DVec3::new(galaxy_radius - 1000.0, 0.0, 0.0); // 1km away
    
    let world_transform = WorldTransform::from_position(world_pos);
    let relative = world_transform.to_camera_relative(camera_pos);
    
    // Should maintain meter precision even at galaxy scale
    assert!((relative.position.x - 1000.0).abs() < 0.1);
}

#[test]
fn test_hierarchical_galaxy_coordinates() {
    let sector_size = 1e18; // 1 million light years per sector
    let world_pos = DVec3::new(2.5e18, 1.3e18, -0.7e18);
    
    let galaxy_pos = GalaxyPosition::from_world_position(world_pos, sector_size);
    assert_eq!(galaxy_pos.sector.sector_coords, IVec3::new(2, 1, -1));
    
    let reconstructed = galaxy_pos.to_world_position();
    assert!((reconstructed - world_pos).length() < 1.0); // Sub-meter precision
}
```

### 5. Galaxy Scale Scene Example

```json
// game/assets/scenes/galaxy_scale_test.json
{
  "entities": [
    {
      "components": {
        "WorldTransform": {
          "position": [9.46e20, 0.0, 0.0],
          "rotation": [0.0, 0.0, 0.0, 1.0],
          "scale": [1e15, 1e15, 1e15]
        },
        "MeshRenderer": { "mesh": "Sphere" },
        "Material": { "color": [1.0, 0.8, 0.0, 1.0] },
        "Name": { "name": "Galaxy Edge Star System" }
      }
    },
    {
      "components": {
        "WorldTransform": {
          "position": [4.73e20, 2.84e20, 0.0],
          "rotation": [0.0, 0.0, 0.0, 1.0],
          "scale": [1e14, 1e14, 1e14]
        },
        "MeshRenderer": { "mesh": "Cube" },
        "Material": { "color": [0.0, 0.5, 1.0, 1.0] },
        "Name": { "name": "Galaxy Core Region (50k ly)" }
      }
    }
  ]
}
```

### Task Order
1. Implement GalaxySector and GalaxyPosition structures
2. Add galaxy coordinate conversion methods to WorldTransform
3. Create galaxy-scale camera configuration
4. Enhance logarithmic depth coefficient calculation
5. Add comprehensive galaxy-scale tests
6. Create galaxy-scale scene examples
7. Update documentation with galaxy-scale best practices
8. Performance profiling and optimization

---

## Context and References

### Existing Implementation
- `engine/src/core/coordinates/world_transform.rs` - Current WorldTransform with f64 positions
- `engine/src/core/coordinates/origin_manager.rs` - Origin shifting for single-player
- `engine/src/core/camera.rs` - Camera with logarithmic depth support
- `engine/src/shaders/logarithmic_depth.wgsl` - Logarithmic depth shader implementation

### External Resources
- https://en.wikipedia.org/wiki/Double-precision_floating-point_format - f64 max ~1.8×10^308
- https://en.wikipedia.org/wiki/Milky_Way - Galaxy dimensions and scale
- https://gameprogrammingpatterns.com/spatial-partition.html - Hierarchical spatial partitioning
- https://outerra.blogspot.com/2012/11/maximizing-depth-buffer-range-and.html - Logarithmic depth theory

### Key Measurements
- 1 light-year = 9.46×10^15 meters
- Milky Way diameter = ~100,000 light-years = ~9.46×10^20 meters
- Galaxy sector size suggestion = 10^18 meters (allows ~1000x1000x1000 sectors)
- f64 precision at 10^21: ~10^5 meters (100km) - requires hierarchical approach

### Gotchas
- Even f64 loses precision at galaxy scale - hierarchical coordinates required
- Logarithmic depth coefficient calculation needs adjustment for extreme ratios
- Performance implications of coordinate transformations at extreme scales
- Sector boundary crossings need careful handling
- Integer overflow possible with naive sector calculations

---

## Validation Gates

```bash
# Format and lint
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run all tests including new galaxy-scale tests
cargo test --workspace

# Specific coordinate tests
cargo test -p engine galaxy_scale
cargo test -p engine hierarchical_galaxy

# Documentation
cargo doc --workspace --no-deps --document-private-items

# Performance benchmark (if available)
cargo bench --bench coordinates galaxy_scale

# Full validation
just preflight
```

## Error Handling Strategy
- Validate coordinate ranges before conversions
- Handle sector boundary crossings gracefully
- Fallback to regular WorldTransform for non-galaxy scales
- Log warnings for extreme coordinate values
- Provide clear error messages for precision loss scenarios

---

## Score: 8/10

### Confidence Assessment
- **Strengths**: 
  - Builds on existing proven implementation
  - Clear hierarchical approach based on research
  - Comprehensive test coverage planned
  - Backward compatible design
  
- **Risks**:
  - Performance at extreme scales needs validation
  - Sector boundary handling complexity
  - Logarithmic depth shader modifications may need iteration

The implementation path is clear with good existing foundation. Main challenges are performance validation and sector boundary handling.