name: "Refactor Assets and Engine API"
description: |
  Move assets to game crate and create minimal engine initialization API for cleaner separation

---

## Goal
Refactor the project structure to move assets from the root directory to the game crate and create a minimal engine initialization API that reduces the game's main.rs from ~150 lines to ~20 lines of simple initialization code.

## Why
- **Clean Separation**: Game crate should contain game-specific assets and minimal engine interaction
- **Maintainability**: Complex engine initialization should be encapsulated within the engine crate
- **Usability**: Simple API makes it easier to create new games/demos with the engine
- **Asset Organization**: Assets belong with the game that uses them, not at project root

## What
Create a minimal engine initialization API and move assets to appropriate locations while maintaining all functionality.

### Success Criteria
- [ ] Assets moved from `/assets/` to `/game/assets/`
- [ ] Script paths are configurable (no hardcoded "assets/scripts/")
- [ ] Engine provides `EngineApp::new()` for simple initialization
- [ ] Game main.rs reduced to ~20 lines using new API
- [ ] All tests pass (`just preflight` succeeds)
- [ ] Editor feature continues to work correctly
- [ ] Hot-reload functionality preserved

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Current codebase patterns
- file: /engine/src/scripting/components.rs:23
  why: Hardcoded script path that needs to be configurable
  pattern: format!("assets/scripts/{}.rhai", self.name)

- file: /engine/src/scripting/script.rs:22
  why: Second hardcoded script path location
  pattern: format!("assets/scripts/{name}.rhai")

- file: /game/src/main.rs:64-208
  why: Complex initialization that needs to be simplified
  pattern: Shows current 150-line manual setup process

- file: /engine/src/io/scene.rs
  why: Scene loading is already path-flexible (good pattern)
  pattern: Scene::load_from_file<P: AsRef<Path>>(path)

- url: https://docs.rs/wgpu/latest/wgpu/
  why: WebGPU initialization patterns for engine API
  section: Instance and Device creation

- url: https://github.com/bevyengine/bevy/tree/main/examples
  why: Reference engine/game separation patterns
  critical: How engine provides simple entry points
```

### Current Codebase Tree
```bash
webgpu-template/
├── assets/                    # TO MOVE to game/assets/
│   ├── scenes/
│   │   ├── demo_scene.json
│   │   └── scripted_demo.json
│   └── scripts/
│       ├── fly_camera.rhai
│       ├── rotating_cube.rhai
│       └── simple_rotate.rhai
├── engine/                    # Core engine library
│   ├── src/
│   │   ├── scripting/         # Hardcoded paths HERE
│   │   ├── io/               # Flexible scene loading
│   │   └── lib.rs
├── game/                     # Game binary crate
│   └── src/
│       └── main.rs           # COMPLEX 150-line init HERE
├── editor/                   # Optional editor
└── justfile                  # Build commands
```

### Desired Codebase Tree
```bash
webgpu-template/
├── engine/
│   ├── src/
│   │   ├── app.rs            # NEW: EngineApp initialization API
│   │   ├── config.rs         # NEW: Asset path configuration
│   │   ├── scripting/        # MODIFIED: Configurable paths
│   │   └── lib.rs            # MODIFIED: Export new API
├── game/
│   ├── assets/               # MOVED: From root level
│   │   ├── scenes/
│   │   └── scripts/
│   └── src/
│       └── main.rs           # SIMPLIFIED: ~20 lines using EngineApp
├── editor/                   # Unchanged
└── justfile                  # Unchanged
```

### Known Gotchas & Library Quirks
```rust
// CRITICAL: Two hardcoded script path locations must be updated
// File: /engine/src/scripting/components.rs:23
impl ScriptRef {
    fn path(&self) -> String {
        format!("assets/scripts/{}.rhai", self.name) // HARDCODED!
    }
}

// File: /engine/src/scripting/script.rs:22
impl Script {
    pub fn from_name(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let path = format!("assets/scripts/{name}.rhai"); // HARDCODED!
    }
}

// CRITICAL: RenderContext::new() is async and requires blocking
let render_context = pollster::block_on(RenderContext::new(instance, None))?;

// CRITICAL: Editor feature flag affects initialization significantly
#[cfg(feature = "editor")]
let editor_state = EditorState::new(/* complex params */);

// CRITICAL: Must use tracing, never println!
use tracing::{debug, error, info, warn, trace};

// CRITICAL: Asset validation depends on proper scene file paths
Scene::load_from_file_with_validation(path, asset_manager)?;
```

## Implementation Blueprint

### Data Models and Structure
Create configuration types for flexible asset handling:
```rust
// engine/src/config.rs
#[derive(Debug, Clone)]
pub struct AssetConfig {
    pub asset_root: PathBuf,
    pub scripts_dir: String,
    pub scenes_dir: String,
}

impl AssetConfig {
    pub fn script_path(&self, name: &str) -> PathBuf {
        self.asset_root.join(&self.scripts_dir).join(format!("{name}.rhai"))
    }
    
    pub fn scene_path(&self, name: &str) -> PathBuf {
        self.asset_root.join(&self.scenes_dir).join(format!("{name}.json"))
    }
}

// engine/src/app.rs
pub struct EngineConfig {
    pub window_title: String,
    pub window_size: Option<(u32, u32)>,
    pub asset_config: AssetConfig,
    pub enable_editor: bool,
    pub enable_scripting: bool,
}

pub struct EngineApp {
    pub window_manager: WindowManager,
    pub render_context: Arc<RenderContext>,
    pub renderer: Renderer,
    pub world: World,
    pub script_engine: Option<ScriptEngine>,
    pub input_state: InputState,
    #[cfg(feature = "editor")]
    pub editor_state: Option<EditorState>,
    // private fields...
}
```

### List of Tasks (Implementation Order)

```yaml
Task 1: Create Asset Configuration System
MODIFY engine/src/lib.rs:
  - ADD module declaration: pub mod config;
  - EXPOSE AssetConfig in prelude

CREATE engine/src/config.rs:
  - IMPLEMENT AssetConfig struct with path methods
  - PROVIDE default() that works with current paths
  - ADD validation for path existence

Task 2: Make Script Paths Configurable  
MODIFY engine/src/scripting/components.rs:
  - FIND: impl ScriptRef { fn path(&self) -> String
  - REPLACE: Accept AssetConfig parameter
  - CHANGE: format!("assets/scripts/{}.rhai") to use config.script_path()

MODIFY engine/src/scripting/script.rs:
  - FIND: pub fn from_name(name: &str) -> Result<Self, Box<dyn std::error::Error>>
  - ADD: AssetConfig parameter 
  - REPLACE: Hardcoded path with config.script_path(name)

Task 3: Update Script Engine Initialization
MODIFY engine/src/scripting/system.rs:
  - FIND: initialize_script_engine function
  - ADD: AssetConfig parameter to pass through to script loading
  - UPDATE: All script loading calls to use configured paths

Task 4: Create Minimal Engine API
CREATE engine/src/app.rs:
  - IMPLEMENT EngineConfig with sensible defaults
  - IMPLEMENT EngineApp::new() for simple initialization
  - IMPLEMENT EngineApp::with_config() for customization
  - HANDLE async RenderContext creation internally
  - SUPPORT conditional editor compilation cleanly
  - PROVIDE update() and render() methods for game loop

Task 5: Move Assets to Game Crate
EXECUTE filesystem operations:
  - CREATE game/assets/ directory
  - MOVE assets/scenes/ to game/assets/scenes/
  - MOVE assets/scripts/ to game/assets/scripts/
  - REMOVE empty assets/ directory

Task 6: Configure Game Asset Paths
MODIFY game/src/main.rs:
  - SETUP AssetConfig pointing to game/assets/
  - REPLACE complex initialization with EngineApp::with_config()
  - REDUCE main function to ~20 lines
  - PRESERVE demo scene creation functionality

Task 7: Update Examples and Tests
MODIFY examples/scene_demo.rs:
  - UPDATE asset paths to use new configuration
  - USE new EngineApp API for initialization

UPDATE any test files:
  - CHANGE hardcoded asset paths to use configuration
  - ENSURE temporary test files still work
```

### Per Task Pseudocode

```rust
// Task 1: Asset Configuration
pub struct AssetConfig {
    pub asset_root: PathBuf,
    pub scripts_dir: String,
    pub scenes_dir: String,
}

impl AssetConfig {
    pub fn script_path(&self, name: &str) -> PathBuf {
        // PATTERN: Always validate name first (no path traversal)
        if name.contains("..") || name.contains("/") {
            panic!("Invalid script name: {name}");
        }
        self.asset_root.join(&self.scripts_dir).join(format!("{name}.rhai"))
    }
}

// Task 4: Engine API Core Structure
impl EngineApp {
    pub fn new(event_loop: &ActiveEventLoop) -> Result<Self, Box<dyn std::error::Error>> {
        // PATTERN: Use default config for simple cases
        let config = EngineConfig::default();
        Self::with_config(event_loop, config)
    }
    
    pub fn with_config(event_loop: &ActiveEventLoop, config: EngineConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // CRITICAL: Initialize logging first (from CLAUDE.md)
        init_logging();
        
        // PATTERN: Determine window size (existing logic from main.rs)
        let (width, height) = config.window_size.unwrap_or_else(|| {
            get_primary_monitor_size(event_loop)
        });
        
        // CRITICAL: Handle async RenderContext with pollster::block_on
        let instance = Arc::new(wgpu::Instance::new(&wgpu::InstanceDescriptor::default()));
        let render_context = pollster::block_on(RenderContext::new((*instance).clone(), None))?;
        
        // PATTERN: Initialize components in dependency order (critical!)
        // 1. Window, 2. Graphics, 3. ECS, 4. Scripts, 5. Editor
        let window = create_window(event_loop, &config)?;
        let (window_manager, renderer) = init_graphics(window, instance, render_context)?;
        let world = World::new();
        let script_engine = if config.enable_scripting {
            Some(init_script_engine_with_config(&config.asset_config))
        } else { None };
        
        Ok(Self { /* ... */ })
    }
}

// Task 6: Simplified Game Main
fn main() {
    let event_loop = EventLoop::builder().build().expect("Failed to create event loop");
    
    let asset_config = AssetConfig {
        asset_root: PathBuf::from("game/assets"),
        scripts_dir: "scripts".to_string(),
        scenes_dir: "scenes".to_string(),
    };
    
    let engine_config = EngineConfig {
        window_title: "WebGPU Game Engine Demo".to_string(),
        asset_config,
        enable_editor: cfg!(feature = "editor"),
        ..Default::default()
    };
    
    let mut app = EngineApp::with_config(&event_loop, engine_config)
        .expect("Failed to initialize engine");
    
    // Setup game-specific scene
    create_demo_scene(&mut app.world, &mut app.renderer);
    
    event_loop.run_app(&mut app).expect("Failed to run app");
}
```

### Integration Points
```yaml
FILESYSTEM:
  - move: "assets/* -> game/assets/*"
  - ensure: "game/assets/ is in .gitignore exceptions if needed"

CONFIG:
  - add to: engine/src/config.rs
  - pattern: "AssetConfig with PathBuf asset_root"

MODULE STRUCTURE:
  - add to: engine/src/lib.rs
  - exports: "pub use app::{EngineApp, EngineConfig};"
  - exports: "pub use config::AssetConfig;"

SCRIPT LOADING:
  - modify: engine/src/scripting/components.rs:23
  - pattern: "Use AssetConfig instead of hardcoded path"
```

## Validation Loop

### Level 1: Syntax & Style
```bash
# Run these FIRST after each task - fix any errors before proceeding
cargo fmt --all                                          # Format code
cargo clippy --workspace --all-targets --all-features -- -D warnings  # Linting with warnings as errors

# Expected: No errors. If errors exist, READ them carefully and fix.
```

### Level 2: Unit Tests for Each New Component
```rust
// CREATE tests for new AssetConfig
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_asset_config_script_path() {
        let config = AssetConfig {
            asset_root: PathBuf::from("game/assets"),
            scripts_dir: "scripts".to_string(),
            scenes_dir: "scenes".to_string(),
        };
        
        let path = config.script_path("test_script");
        assert_eq!(path, PathBuf::from("game/assets/scripts/test_script.rhai"));
    }
    
    #[test]
    #[should_panic]
    fn test_asset_config_rejects_path_traversal() {
        let config = AssetConfig::default();
        config.script_path("../evil"); // Should panic
    }
    
    #[test]
    fn test_engine_app_basic_initialization() {
        // Create minimal test - may need to mock event loop
        // Focus on AssetConfig being passed through correctly
    }
}
```

```bash
# Run after implementing each component:
cargo test --workspace
# If failing: Read error carefully, understand root cause, fix code, re-run
```

### Level 3: Integration Test - Asset Loading
```bash
# After moving assets, verify script loading works:
# 1. Run the game
cargo run --bin game

# 2. Check logs for script loading confirmation
# Expected: "info!(script = "fly_camera", "Script loaded successfully")"
# If error: Check asset paths in logs, verify files moved correctly

# 3. Test with editor feature
cargo run --bin game --features editor

# Expected: Editor opens, viewport shows scene
# If error: Check editor asset loading, verify scene files moved
```

## Final Validation Checklist
- [ ] All tests pass: `cargo test --workspace`
- [ ] No linting errors: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] No formatting issues: `cargo fmt --all -- --check`
- [ ] Documentation builds: `cargo doc --workspace --no-deps --document-private-items`
- [ ] Game runs successfully: `cargo run --bin game`
- [ ] Editor feature works: `cargo run --bin game --features editor`
- [ ] Scripts load from new location (check logs)
- [ ] Scene files load from new location
- [ ] Hot-reload functionality preserved
- [ ] All preflight checks pass: `just preflight`

---

## Anti-Patterns to Avoid
- ❌ Don't break the async RenderContext initialization sequence
- ❌ Don't ignore editor feature flag complexity - handle conditionally  
- ❌ Don't hardcode new asset paths - use configuration everywhere
- ❌ Don't use println! - all logging must use tracing macros
- ❌ Don't skip intermediate validation - test each task before proceeding
- ❌ Don't move assets before making paths configurable
- ❌ Don't assume working directory - always use absolute or configured paths

## Confidence Score: 9/10
This PRP provides comprehensive context including specific file locations, existing patterns to follow, detailed step-by-step implementation with validation at each stage, and handles all the critical gotchas identified in research. The phased approach minimizes risk by making each change independently testable.