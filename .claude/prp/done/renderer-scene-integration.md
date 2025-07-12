name: "Renderer-Scene Integration System - WebGPU Template"
description: |

## Purpose
Implement comprehensive integration between the existing WebGPU renderer and scene serialization system, enabling visual representation of serialized scenes with automatic mesh assignment, material management, and development workflow improvements.

## Core Principles
1. **Backwards Compatibility**: Maintain existing renderer and scene system functionality
2. **String-based Assets**: Use human-readable mesh references for scene files
3. **Graceful Degradation**: Handle missing assets with fallbacks and visual indicators
4. **Development Workflow**: Hot-reloading and debugging tools for rapid iteration
5. **Global rules**: Follow all rules in CLAUDE.md

---

## Goal
Build a complete renderer-scene integration that allows developers to:
- Load scenes with graphics components and render them immediately
- Use string-based mesh references in human-readable scene files
- Automatically assign default meshes to entities based on hierarchy/properties
- Hot-reload scene files during development with immediate visual feedback
- Debug scene rendering with statistics and error visualization

## Why
- **Rapid Prototyping**: Load and visualize scenes without manual mesh assignment
- **Content Creation**: Enable level editors and scene authoring tools
- **Development Workflow**: Fast iteration with hot-reload and debugging tools
- **Asset Management**: Organized, human-readable asset references

## What
Implement renderer-scene integration that handles:
- Graphics component serialization (MeshId, Material) in scene JSON
- String-based mesh asset management with fallback system
- Automatic renderer-world bridge with camera integration
- Hot-reloading infrastructure for development iteration
- Scene debugging and statistics reporting

### Success Criteria
- [ ] Scene files can include MeshId and Material components
- [ ] String-based mesh references work with fallback system
- [ ] renderer.render_world() method works with scene-loaded entities
- [ ] world.assign_default_meshes() automatically sets up renderables
- [ ] Hot-reload watches scene files and updates renderer
- [ ] Debug overlay shows scene statistics and errors

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://sotrh.github.io/learn-wgpu/
  why: WebGPU rendering pipeline fundamentals and best practices
  
- url: https://webgpufundamentals.org/webgpu/lessons/webgpu-scene-graphs.html  
  why: Scene graph architecture patterns for WebGPU
  critical: Hierarchical transform management and rendering optimization
  
- url: https://bevy-cheatbook.github.io/assets/hot-reload.html
  why: Asset hot-reloading patterns in Rust game engines
  section: File watching and reload workflow implementation
  
- url: https://fasterthanli.me/articles/so-you-want-to-live-reload-rust
  why: Comprehensive guide to hot-reloading techniques in Rust
  critical: File watching with notify crate and error handling
  
- url: https://matthewmacfarquhar.medium.com/webgpu-rendering-part-19-loading-materials-839df44e0410
  why: WebGPU material management and pipeline integration
  section: PBR material handling and texture loading

- file: engine/src/graphics/renderer.rs
  why: Current renderer implementation with mesh cache and ECS integration
  
- file: engine/src/graphics/material.rs
  why: Material component with existing serde support
  
- file: engine/src/graphics/mesh.rs
  why: Mesh generation and GPU data management patterns
  
- file: engine/src/io/scene.rs
  why: Scene serialization system and component extension patterns
  
- file: engine/src/core/entity/world.rs
  why: World wrapper patterns and ECS helper methods
```

### Current Codebase tree  
```bash
webgpu-template/
├── engine/
│   ├── src/
│   │   ├── graphics/
│   │   │   ├── renderer.rs       # Current renderer with mesh cache, ECS integration
│   │   │   ├── material.rs       # Material component (has serde derives)
│   │   │   ├── mesh.rs           # Mesh generation (cube, sphere, plane)
│   │   │   ├── pipeline.rs       # WebGPU render pipeline
│   │   │   └── context.rs        # WebGPU device/queue management
│   │   ├── io/
│   │   │   ├── scene.rs          # Scene serialization (hardcoded components)
│   │   │   ├── entity_mapper.rs  # Entity ID remapping
│   │   │   └── component_registry.rs # Dynamic component system
│   │   ├── core/
│   │   │   ├── entity/
│   │   │   │   ├── world.rs      # World wrapper with helper methods
│   │   │   │   └── components.rs # Transform, Parent, GlobalTransform
│   │   │   └── camera.rs         # Camera component (has serde derives)
│   │   └── lib.rs                # Public exports
├── assets/scenes/                # Demo scenes with current format
└── examples/scene_demo.rs        # Scene creation and loading example
```

### Desired Codebase tree with files to be added
```bash
engine/
├── src/
│   ├── graphics/
│   │   ├── renderer.rs           # MODIFY: Add render_world(), string-based mesh cache
│   │   ├── asset_manager.rs      # NEW: String-based mesh library and fallbacks
│   │   └── mesh_library.rs       # NEW: Default mesh collection (cube, sphere, plane)
│   ├── io/
│   │   ├── scene.rs              # MODIFY: Add MeshId and Material serialization
│   │   ├── hot_reload.rs         # NEW: File watching and scene hot-reload
│   │   └── scene_debug.rs        # NEW: Scene statistics and debug visualization
│   ├── core/entity/
│   │   ├── world.rs              # MODIFY: Add assign_default_meshes(), render helpers
│   │   └── components.rs         # MODIFY: Update MeshId to string-based
│   └── dev/                      # NEW MODULE: Development tools
│       ├── mod.rs                # Development utilities export
│       └── debug_overlay.rs      # Scene debugging overlay
```

### Known Gotchas & Library Quirks
```rust
// CRITICAL: MeshId currently uses u64, must change to String for scene serialization
// Current: #[derive(Debug, Clone, Copy)] pub struct MeshId(pub u64);
// Needed:  #[derive(Debug, Clone, Serialize, Deserialize)] pub struct MeshId(pub String);

// CRITICAL: Renderer mesh cache uses HashMap<u64, MeshGpuData>
// Must change to HashMap<String, MeshGpuData> for string-based lookups

// CRITICAL: notify crate for file watching requires tokio or async-std
// Use notify = "6.0" with RecommendedWatcher for cross-platform support

// GOTCHA: Hot-reload must handle file write intermediates (editors create temp files)
// Use debounce timer to avoid multiple reload triggers

// PATTERN: Scene component serialization is hardcoded in scene.rs match statements
// Must add MeshId and Material cases to both from_world() and instantiate()

// PATTERN: World helper methods follow naming convention spawn_with_*, add_*
// New methods should follow: assign_default_meshes(), get_scene_stats()

// PERFORMANCE: Object uniforms recreated every frame in current renderer
// Consider caching for hot-reload scenarios with many entities
```

## Implementation Blueprint

### Data models and structure

```rust
// engine/src/graphics/renderer.rs - Updated mesh cache
pub struct Renderer {
    // Change from HashMap<u64, MeshGpuData> to:
    mesh_cache: HashMap<String, MeshGpuData>,
    // Add mesh library for defaults
    default_meshes: MeshLibrary,
}

// engine/src/graphics/asset_manager.rs - NEW
pub struct MeshLibrary {
    // Predefined mesh names and their generation functions
    generators: HashMap<String, Box<dyn Fn() -> Mesh>>,
}

// engine/src/graphics/renderer.rs - Updated MeshId
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MeshId(pub String);

// engine/src/io/scene.rs - Scene with graphics components
// Components map should include:
// "MeshId": {"id": "cube"}
// "Material": {"base_color": [1.0, 0.5, 0.2, 1.0], "metallic": 0.0, "roughness": 0.5}

// engine/src/io/hot_reload.rs - NEW
pub struct SceneWatcher {
    watcher: RecommendedWatcher,
    scene_path: PathBuf,
    callback: Box<dyn Fn(&mut World, &mut Renderer) -> Result<(), Box<dyn Error>>>,
}

// engine/src/dev/debug_overlay.rs - NEW  
pub struct SceneStats {
    pub entity_count: usize,
    pub renderable_count: usize,
    pub camera_count: usize,
    pub mesh_types: HashMap<String, usize>,
    pub material_count: usize,
}
```

### List of tasks to be completed in order

```yaml
Task 1: Convert MeshId to string-based system
MODIFY engine/src/graphics/renderer.rs:
  - CHANGE MeshId(u64) to MeshId(String)  
  - UPDATE mesh_cache from HashMap<u64, MeshGpuData> to HashMap<String, MeshGpuData>
  - ADD serde derives to MeshId struct
  - UPDATE upload_mesh() to return string-based MeshId
  
CREATE engine/src/graphics/mesh_library.rs:
  - CREATE MeshLibrary struct with default mesh generators
  - IMPLEMENT get_or_generate() for lazy mesh creation
  - PROVIDE predefined meshes: "cube", "sphere", "plane", "error_mesh"

Task 2: Add graphics components to scene serialization
MODIFY engine/src/io/scene.rs:
  - ADD MeshId serialization case in from_world() method
  - ADD Material serialization case in from_world() method  
  - ADD MeshId deserialization case in instantiate() method
  - ADD Material deserialization case in instantiate() method
  - HANDLE missing mesh fallback to "error_mesh"

MODIFY engine/src/io/component_registry.rs:
  - ADD MeshId to with_default_components()
  - ADD Material to with_default_components()

Task 3: Implement renderer-world bridge
MODIFY engine/src/graphics/renderer.rs:
  - ADD render_world(&self, world: &World, camera_entity: Entity) method
  - ADD mesh assignment integration with MeshLibrary
  - UPDATE existing render() to use new string-based cache
  - ADD camera selection and matrix calculation from world

MODIFY engine/src/core/entity/world.rs:
  - ADD assign_default_meshes(&mut self, renderer: &mut Renderer) method  
  - ADD get_scene_stats(&self) -> SceneStats method
  - ADD clear_graphics_components(&mut self) helper method
  - IMPLEMENT mesh assignment strategies (hierarchy-based, name-based)

Task 4: Asset management and error handling
CREATE engine/src/graphics/asset_manager.rs:
  - IMPLEMENT AssetManager with mesh resolution and fallbacks
  - ADD validate_scene_assets() for asset dependency checking
  - ADD error reporting with entity context
  - PROVIDE default material and mesh fallbacks

MODIFY engine/src/io/scene.rs:
  - ADD asset validation during scene loading
  - ADD warning logging for missing assets
  - ADD graceful degradation for invalid materials/meshes

Task 5: Hot-reload infrastructure  
CREATE engine/src/io/hot_reload.rs:
  - IMPLEMENT SceneWatcher with notify crate
  - ADD file change detection with debouncing
  - ADD reload callback system
  - HANDLE file write intermediates and error recovery

CREATE engine/src/dev/debug_overlay.rs:
  - IMPLEMENT scene statistics collection
  - ADD debug visualization helpers
  - ADD performance monitoring (draw calls, entity counts)
  - ADD error visualization (red wireframe for missing assets)

Task 6: Development workflow integration
MODIFY engine/src/core/entity/world.rs:
  - ADD watch_scene() method for hot-reload setup
  - ADD debug helpers for scene inspection
  - INTEGRATE with existing logging system

CREATE engine/src/dev/mod.rs:
  - EXPORT development tools and debug utilities
  - ADD feature flag for debug builds only
  - INTEGRATE with existing engine structure
```

### Per task pseudocode

```rust
// Task 1 - String-based MeshId conversion
impl Renderer {
    pub fn upload_mesh(&mut self, mesh: Mesh, name: &str) -> MeshId {
        let mesh_id = MeshId(name.to_string());
        let gpu_data = MeshGpuData::from_mesh(&self.context, mesh);
        self.mesh_cache.insert(name.to_string(), gpu_data);
        mesh_id
    }
    
    fn get_or_create_mesh(&mut self, mesh_id: &MeshId) -> &MeshGpuData {
        if !self.mesh_cache.contains_key(&mesh_id.0) {
            // Generate default mesh or fallback to error mesh
            let mesh = self.mesh_library.get_or_generate(&mesh_id.0)
                       .unwrap_or_else(|| self.mesh_library.error_mesh());
            self.upload_mesh(mesh, &mesh_id.0);
        }
        &self.mesh_cache[&mesh_id.0]
    }
}

// Task 3 - Renderer-world bridge  
impl Renderer {
    pub fn render_world(&mut self, world: &World, camera_entity: Entity) -> Result<()> {
        // Get camera and calculate view-projection matrix
        let (camera, camera_transform) = world.query_one_mut::<(&Camera, &GlobalTransform)>(camera_entity)?;
        let view_proj = camera.view_projection_matrix(camera_transform);
        
        // Update camera uniforms
        self.update_camera_uniforms(view_proj);
        
        // Query all renderable entities
        let mut render_query = world.query::<(&MeshId, &Material, &GlobalTransform)>();
        for (entity, (mesh_id, material, transform)) in render_query.iter() {
            // Get or create mesh GPU data
            let mesh_data = self.get_or_create_mesh(mesh_id);
            
            // Create object uniforms
            let object_uniforms = ObjectUniforms {
                model_matrix: transform.matrix,
                material: material.clone(),
            };
            
            // Draw the entity
            self.draw_entity(mesh_data, object_uniforms);
        }
        
        Ok(())
    }
}

// Task 2 - Scene serialization extension
impl Scene {
    pub fn from_world(world: &World) -> Self {
        // ... existing code for Transform, Camera, Parent
        
        // Add MeshId serialization
        if let Ok(mesh_id) = world.get::<MeshId>(entity) {
            match serde_json::to_value(*mesh_id) {
                Ok(value) => { components.insert("MeshId".to_string(), value); }
                Err(e) => { error!(error = %e, "Failed to serialize MeshId"); }
            }
        }
        
        // Add Material serialization  
        if let Ok(material) = world.get::<Material>(entity) {
            match serde_json::to_value(*material) {
                Ok(value) => { components.insert("Material".to_string(), value); }
                Err(e) => { error!(error = %e, "Failed to serialize Material"); }
            }
        }
    }
    
    pub fn instantiate(&self, world: &mut World) -> Result<EntityMapper> {
        // ... existing code for Transform, Camera, Parent
        
        "MeshId" => {
            match serde_json::from_value::<MeshId>(value.clone()) {
                Ok(mesh_id) => {
                    // Validate mesh exists or use fallback
                    let final_mesh_id = validate_mesh_id(&mesh_id.0)
                                       .unwrap_or_else(|| MeshId("error_mesh".to_string()));
                    world.insert_one(entity, final_mesh_id)?;
                }
                Err(e) => { warn!(error = %e, "Failed to deserialize MeshId"); }
            }
        }
        
        "Material" => {
            match serde_json::from_value::<Material>(value.clone()) {
                Ok(material) => {
                    world.insert_one(entity, material)?;
                }
                Err(e) => { warn!(error = %e, "Failed to deserialize Material"); }
            }
        }
    }
}

// Task 5 - Hot-reload implementation
impl SceneWatcher {
    pub fn new<F>(scene_path: &Path, mut callback: F) -> Result<Self> 
    where F: Fn(&mut World, &mut Renderer) -> Result<()> + 'static 
    {
        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(tx, Duration::from_millis(100))?;
        watcher.watch(scene_path, RecursiveMode::NonRecursive)?;
        
        // Spawn background thread for file events
        thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                match event {
                    DebouncedEvent::Write(_) | DebouncedEvent::Create(_) => {
                        if let Err(e) = callback(world, renderer) {
                            error!(error = %e, "Hot-reload callback failed");
                        }
                    }
                    _ => {}
                }
            }
        });
        
        Ok(SceneWatcher { watcher, scene_path: scene_path.to_owned(), callback: Box::new(callback) })
    }
}
```

### Integration Points
```yaml
RENDERER:
  - String-based mesh cache instead of numeric IDs
  - MeshLibrary integration for default meshes and fallbacks  
  - render_world() method for complete scene rendering
  - Asset validation and error handling
  
SCENE SYSTEM:
  - MeshId and Material component serialization
  - Asset reference validation and fallback handling
  - Hot-reload integration for development workflow
  
WORLD:
  - Mesh assignment automation with assign_default_meshes()
  - Scene statistics and debugging helpers
  - Hot-reload callback integration

ECS COMPONENTS:
  - MeshId component with string-based asset references
  - Material component (already serialization-ready)
  - Existing Transform/Camera integration preserved

ERROR HANDLING:
  - Missing mesh fallback to "error_mesh" (red wireframe cube)
  - Invalid material parameters use default material  
  - Asset validation with detailed error reporting
  - Visual indicators for debugging (debug overlay)
```

## Validation Loop

### Level 1: Syntax & Style
```bash
# Run these FIRST - fix any errors before proceeding
cargo fmt --all                     # Format code
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Expected: No errors or warnings
```

### Level 2: Unit Tests
```rust
// Test string-based MeshId system
#[test]
fn test_string_mesh_id_serialization() {
    let mesh_id = MeshId("cube".to_string());
    let json = serde_json::to_string(&mesh_id).unwrap();
    let deserialized: MeshId = serde_json::from_str(&json).unwrap();
    assert_eq!(mesh_id, deserialized);
}

// Test graphics component scene round-trip
#[test]  
fn test_scene_with_graphics_components() {
    let mut world = World::new();
    
    // Create entity with graphics components
    let entity = world.spawn((
        Transform::default(),
        GlobalTransform::default(),
        MeshId("sphere".to_string()),
        Material::from_rgba(1.0, 0.5, 0.2, 1.0),
    ));
    
    // Save to scene
    let scene = Scene::from_world(&world);
    
    // Load into new world  
    let mut new_world = World::new();
    let mapper = scene.instantiate(&mut new_world).unwrap();
    
    // Verify graphics components preserved
    let new_entity = mapper.remap(0).unwrap();
    assert!(new_world.get::<MeshId>(new_entity).is_ok());
    assert!(new_world.get::<Material>(new_entity).is_ok());
}

// Test renderer-world integration
#[test]
fn test_render_world_integration() {
    let mut world = World::new();
    let mut renderer = Renderer::new(&render_context).unwrap();
    
    // Create camera and renderable entity
    let camera = world.spawn((
        Transform::default(),
        GlobalTransform::default(), 
        Camera::default(),
    ));
    
    world.spawn((
        Transform::default(),
        GlobalTransform::default(),
        MeshId("cube".to_string()),
        Material::default(),
    ));
    
    // Should render without errors
    renderer.render_world(&world, camera).unwrap();
}

// Test mesh fallback system
#[test]
fn test_missing_mesh_fallback() {
    let mut world = World::new();
    let mut renderer = Renderer::new(&render_context).unwrap();
    
    // Entity with non-existent mesh
    world.spawn((
        Transform::default(),
        GlobalTransform::default(),
        MeshId("nonexistent_mesh".to_string()),
        Material::default(),
    ));
    
    // Should assign default meshes without errors
    world.assign_default_meshes(&mut renderer).unwrap();
    
    // Verify fallback mesh was assigned
    let entities_with_error_mesh = world.query::<&MeshId>()
        .iter()
        .filter(|(_, mesh_id)| mesh_id.0 == "error_mesh")
        .count();
    assert_eq!(entities_with_error_mesh, 1);
}
```

```bash
# Run tests iteratively until passing:
cargo test --package engine --lib graphics::tests -v
cargo test --package engine --lib io::tests -v  
cargo test --package engine --lib core::entity::tests -v
# Fix issues, re-run until green
```

### Level 3: Integration Test  
```bash
# Build the project
just preflight

# Test complete workflow
cd assets/scenes
cat > test_graphics_scene.json << 'EOF'
{
  "entities": [
    {
      "components": {
        "Transform": {"position":[0,0,5],"rotation":[0,0,0,1],"scale":[1,1,1]},
        "Camera": {"fov_y_radians":1.047,"aspect_ratio":1.777,"z_near":0.1,"z_far":1000.0,"projection_mode":"Perspective"}
      }
    },
    {
      "components": {
        "Transform": {"position":[2,0,0],"rotation":[0,0,0,1],"scale":[1,1,1]},
        "MeshId": {"id": "cube"},
        "Material": {"base_color": [1.0, 0.5, 0.2, 1.0]}
      }
    }
  ]
}
EOF

# Test in game code - add to game/src/main.rs:
# let mut world = World::new();
# let mut renderer = Renderer::new(&render_context)?;
# world.load_scene("assets/scenes/test_graphics_scene.json")?;
# world.assign_default_meshes(&mut renderer)?;
# let camera = world.query::<&Camera>().iter().next().unwrap().0;
# renderer.render_world(&world, camera)?;

# Run and verify rendering
just run
```

## Final Validation Checklist
- [ ] All tests pass: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] Formatting clean: `cargo fmt --all -- --check`
- [ ] String-based MeshId serialization works
- [ ] Scene with graphics components round-trip succeeds
- [ ] render_world() method renders scene-loaded entities
- [ ] assign_default_meshes() sets up missing renderables
- [ ] Missing mesh fallback to error_mesh works
- [ ] Hot-reload detects scene file changes
- [ ] Debug overlay shows scene statistics
- [ ] Documentation complete: `cargo doc --workspace --no-deps`

---

## Anti-Patterns to Avoid
- ❌ Don't break existing renderer functionality - maintain backwards compatibility
- ❌ Don't ignore asset validation - always provide fallbacks for missing resources
- ❌ Don't use numeric mesh IDs in scenes - string references only
- ❌ Don't skip hot-reload debouncing - editors create temporary files  
- ❌ Don't recreate GPU resources unnecessarily - cache and reuse when possible
- ❌ Don't panic on missing assets - log warnings and use fallbacks

## Confidence Score: 8/10

The implementation path is clear with strong existing foundations in both renderer and scene systems. Main complexities are the string-based MeshId conversion and hot-reload infrastructure, but established patterns from Bevy and the Rust ecosystem provide solid guidance. Two points deducted for:
1. Hot-reload file watching requires careful handling of editor intermediates and error recovery
2. Performance implications of new rendering workflow need validation under load