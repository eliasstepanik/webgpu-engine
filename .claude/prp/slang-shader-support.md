name: "Slang Shader Language Support Implementation"
description: |

## Purpose
Implement comprehensive Slang shader language support in the WebGPU template engine, enabling developers to write shaders in Slang and compile them to SPIR-V for WebGPU consumption. This includes runtime compilation, hot-reload support, and maintaining backward compatibility with existing WGSL shaders.

## Core Principles
1. **Maintain Backward Compatibility**: Existing WGSL shaders must continue to work
2. **Follow Existing Patterns**: Use the current abstraction patterns from the graphics module
3. **Progressive Enhancement**: Start with basic compilation, then add hot-reload and caching
4. **Robust Error Handling**: Gracefully handle compilation failures with fallback mechanisms
5. **Global rules**: Follow all rules in CLAUDE.md, especially regarding logging with tracing

---

## Goal
Build a flexible shader system that supports both WGSL and Slang shader languages with runtime compilation, hot-reloading, and proper error handling. The system should seamlessly integrate with the existing WebGPU renderer while providing a path for future shader language additions.

## Why
- **Modern Shader Development**: Slang provides advanced features like generics, interfaces, and automatic differentiation
- **Developer Experience**: Hot-reload capability speeds up shader iteration
- **Future-Proofing**: Abstraction layer allows easy addition of other shader languages
- **Performance**: Compiled shader caching reduces startup time
- **Compatibility**: SPIR-V output works across different graphics APIs

## What
Implement a shader management system that:
- Compiles Slang shaders to SPIR-V at runtime
- Supports hot-reloading of shader files during development
- Maintains compatibility with existing WGSL shaders
- Provides clear compilation error messages
- Caches compiled shaders for performance
- Allows multiple shader pipelines

### Success Criteria
- [ ] Slang shaders compile successfully to SPIR-V
- [ ] Hot-reload works for both Slang and WGSL shaders
- [ ] Existing WGSL shaders continue to work without modification
- [ ] Compilation errors are logged with helpful messages
- [ ] Shader cache reduces subsequent load times
- [ ] Multiple pipelines can be created with different shaders
- [ ] All tests pass and no performance regression

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://github.com/shader-slang/slang
  why: Official Slang documentation for language features and compiler API
  
- url: https://github.com/FloatyMonkey/slang-rs
  why: Rust bindings we'll use - shows compilation pipeline and API usage
  
- url: https://www.w3.org/TR/webgpu/#shader-modules
  why: WebGPU shader module requirements - SPIR-V must meet these specs
  
- file: engine/src/graphics/pipeline.rs
  why: Current pipeline creation pattern - we must follow this structure
  
- file: engine/src/io/hot_reload.rs
  why: Existing hot-reload implementation using notify crate - reuse this pattern
  
- file: engine/src/shaders/mod.rs
  why: Current shader loading approach - maintain compatibility
  
- file: engine/src/graphics/renderer.rs
  why: How shaders are used in rendering - integration points
  
- doc: https://github.com/shader-slang/slang/blob/master/docs/user-guide/index.md
  section: Compilation API
  critical: Must use session-based compilation for proper error handling
```

### Current Codebase Structure
```
engine/src/
├── core/           # ECS and math
├── graphics/       # Rendering systems
│   ├── pipeline.rs # RenderPipeline creation
│   ├── renderer.rs # Main renderer
│   └── uniform.rs  # Uniform buffer traits
├── io/             # File I/O and hot-reload
│   └── hot_reload.rs # Notify-based file watcher
├── shaders/        # Shader code
│   ├── mod.rs      # Embedded WGSL constant
│   └── basic.wgsl  # Default shader
└── lib.rs          # Module exports
```

### Desired Codebase Structure with New Files
```
engine/src/
├── graphics/
│   ├── shader_compiler/ # NEW: Shader compilation module
│   │   ├── mod.rs      # Public API and traits
│   │   ├── slang.rs    # Slang-specific compiler
│   │   ├── wgsl.rs     # WGSL pass-through compiler
│   │   └── cache.rs    # Compiled shader cache
│   ├── shader_manager.rs # NEW: Runtime shader management
│   └── pipeline_builder.rs # NEW: Flexible pipeline creation
├── shaders/
│   ├── slang/          # NEW: Slang shader files
│   │   └── basic.slang # Slang version of basic shader
│   └── mod.rs          # Updated to support multiple formats
```

### Known Gotchas & Library Quirks
```rust
// CRITICAL: slang-rs requires Slang SDK installation
// The SDK must be available via SLANG_DIR env var or in system PATH

// CRITICAL: WebGPU requires SPIR-V to use specific capabilities
// Must compile Slang with target profile that matches WebGPU requirements

// CRITICAL: Hot-reload file watching can cause issues on some filesystems
// Use debouncing (already implemented in existing hot_reload.rs)

// CRITICAL: Shader compilation is expensive - always cache results
// Use content hash as cache key, not just filename

// CRITICAL: wgpu ShaderModule creation can fail at runtime
// Always have a fallback shader ready (use existing BASIC_SHADER)

// Pattern: All logging must use tracing crate, NOT println!
use tracing::{debug, error, info, warn};

// Pattern: Error handling uses Result<T, Box<dyn Error>>
// Follow existing patterns in io module
```

## Implementation Blueprint

### Core Types and Traits

```rust
// engine/src/graphics/shader_compiler/mod.rs

/// Supported shader languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderLanguage {
    Wgsl,
    Slang,
}

/// Compiled shader result
pub struct CompiledShader {
    pub spirv: Option<Vec<u32>>,  // For Slang
    pub wgsl: Option<String>,      // For WGSL
    pub entry_points: Vec<String>,
    pub language: ShaderLanguage,
}

/// Trait for shader compilers
pub trait ShaderCompiler: Send + Sync {
    fn compile(&self, source: &str, filename: &str) -> Result<CompiledShader, Box<dyn Error>>;
    fn language(&self) -> ShaderLanguage;
}

// engine/src/graphics/shader_manager.rs

pub struct ShaderManager {
    compilers: HashMap<ShaderLanguage, Box<dyn ShaderCompiler>>,
    cache: ShaderCache,
    watcher: Option<ShaderWatcher>,
}

pub struct ShaderHandle {
    id: u64,
    language: ShaderLanguage,
    path: Option<PathBuf>,
}
```

### Task List (Ordered for Progressive Success)

```yaml
Task 1: Create shader compiler trait and module structure
MODIFY engine/src/graphics/mod.rs:
  - ADD: pub mod shader_compiler;
  - ADD: pub mod shader_manager;
  - PRESERVE: existing module exports

CREATE engine/src/graphics/shader_compiler/mod.rs:
  - DEFINE: ShaderLanguage enum
  - DEFINE: CompiledShader struct
  - DEFINE: ShaderCompiler trait
  - EXPORT: public types

Task 2: Implement WGSL compiler (passthrough)
CREATE engine/src/graphics/shader_compiler/wgsl.rs:
  - IMPLEMENT: ShaderCompiler for WgslCompiler
  - PATTERN: Simply wrap source in CompiledShader with wgsl field
  - LOG: Using tracing::debug for compilation events

Task 3: Add Slang dependency and implement Slang compiler
MODIFY engine/Cargo.toml:
  - ADD: slang = { git = "https://github.com/FloatyMonkey/slang-rs.git" }
  - ADD: optional = true flag for slang feature
  - UPDATE: default features to include "slang"

CREATE engine/src/graphics/shader_compiler/slang.rs:
  - IMPLEMENT: ShaderCompiler for SlangCompiler
  - USE: slang::GlobalSession for compilation
  - TARGET: SPIR-V with Vulkan 1.1 profile
  - ERROR: Map Slang errors to readable messages
  - LOG: Compilation time and success/failure

Task 4: Implement shader cache
CREATE engine/src/graphics/shader_compiler/cache.rs:
  - USE: HashMap<u64, CompiledShader> for in-memory cache
  - KEY: Hash of (source_content + compiler_version + options)
  - OPTIONAL: Disk cache in user cache directory
  - PATTERN: Similar to mesh_cache in renderer.rs

Task 5: Create shader manager
CREATE engine/src/graphics/shader_manager.rs:
  - INTEGRATE: All compilers into unified interface
  - DETECT: Language by file extension (.wgsl, .slang)
  - COMPILE: On-demand with cache lookup
  - STORE: Active shaders by handle
  - PATTERN: Similar structure to AssetManager

Task 6: Add hot-reload support
MODIFY engine/src/graphics/shader_manager.rs:
  - REUSE: Pattern from io/hot_reload.rs
  - WATCH: assets/shaders directory
  - TRIGGER: Recompilation on file change
  - NOTIFY: Renderer to recreate affected pipelines
  - DEBOUNCE: 300ms like existing implementation

Task 7: Create pipeline builder
CREATE engine/src/graphics/pipeline_builder.rs:
  - BUILDER: Pattern for flexible pipeline creation
  - ACCEPT: ShaderHandle instead of hardcoded shader
  - MAINTAIN: Compatibility with existing bind group layouts
  - PATTERN: Follow existing RenderPipeline::new_basic_3d

Task 8: Update renderer to use shader manager
MODIFY engine/src/graphics/renderer.rs:
  - ADD: shader_manager: ShaderManager field
  - CHANGE: Pipeline creation to use ShaderHandle
  - MAINTAIN: Existing rendering logic
  - FALLBACK: Use embedded BASIC_SHADER if compilation fails

Task 9: Convert basic shader to Slang
CREATE engine/src/shaders/slang/basic.slang:
  - PORT: Existing basic.wgsl to Slang syntax
  - MAINTAIN: Same uniform structure and binding points
  - TEST: Renders identical output

Task 10: Add tests
CREATE engine/src/graphics/shader_compiler/tests.rs:
  - TEST: WGSL passthrough compilation
  - TEST: Slang to SPIR-V compilation
  - TEST: Cache hit/miss behavior
  - TEST: Error handling for invalid shaders
  - TEST: Hot-reload file watching
```

### Integration Pseudocode

```rust
// Task 1: Shader compiler trait
// engine/src/graphics/shader_compiler/mod.rs
pub trait ShaderCompiler {
    fn compile(&self, source: &str, filename: &str) -> Result<CompiledShader, Box<dyn Error>> {
        // LOG: info!(filename = filename, "Compiling shader");
        // TIME: Start timer for compilation
        // COMPILE: Language-specific implementation
        // LOG: debug!(duration = ?elapsed, "Shader compiled");
        // RETURN: CompiledShader or error with context
    }
}

// Task 3: Slang compiler
// engine/src/graphics/shader_compiler/slang.rs
impl ShaderCompiler for SlangCompiler {
    fn compile(&self, source: &str, filename: &str) -> Result<CompiledShader, Box<dyn Error>> {
        // CREATE: Global session (cached)
        // OPTIONS: OptimizationLevel::High, matrix_layout_row(true)
        // TARGET: SPIR-V Vulkan 1.1 profile
        // LOAD: Module from source
        // FIND: Entry points (vertexMain, fragmentMain)
        // LINK: Program
        // GET: SPIR-V bytecode
        // RETURN: CompiledShader with spirv field
    }
}

// Task 5: Shader manager
// engine/src/graphics/shader_manager.rs
impl ShaderManager {
    pub fn load_shader(&mut self, path: &Path) -> Result<ShaderHandle, Box<dyn Error>> {
        // DETECT: Language from extension
        // CHECK: Cache for existing compilation
        // READ: File contents
        // COMPILE: Using appropriate compiler
        // CACHE: Result
        // REGISTER: For hot-reload if development build
        // RETURN: Handle
    }
    
    pub fn get_compiled(&self, handle: &ShaderHandle) -> Option<&CompiledShader> {
        // LOOKUP: In cache by handle
        // RETURN: Compiled shader data
    }
}

// Task 8: Renderer integration
// engine/src/graphics/renderer.rs
impl Renderer {
    pub fn create_pipeline_from_shader(&mut self, shader_handle: ShaderHandle) -> Result<(), Box<dyn Error>> {
        // GET: Compiled shader from manager
        // CREATE: ShaderModule based on format
        //   - WGSL: Direct creation
        //   - SPIR-V: Use create_shader_module_spirv
        // BUILD: Pipeline using PipelineBuilder
        // STORE: In pipeline cache
    }
}
```

## Validation Loop

### Level 1: Syntax & Build
```bash
# After each task, run:
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --workspace

# Expected: No errors. Fix any issues before proceeding.
```

### Level 2: Unit Tests
```rust
// Add to each new module's tests.rs or inline

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wgsl_passthrough() {
        let compiler = WgslCompiler::new();
        let source = "// Simple WGSL shader";
        let result = compiler.compile(source, "test.wgsl").unwrap();
        assert_eq!(result.language, ShaderLanguage::Wgsl);
        assert_eq!(result.wgsl.unwrap(), source);
    }
    
    #[test]
    fn test_slang_compilation() {
        // Skip if Slang not available
        if std::env::var("SLANG_DIR").is_err() {
            return;
        }
        
        let compiler = SlangCompiler::new().unwrap();
        let source = include_str!("test_data/simple.slang");
        let result = compiler.compile(source, "simple.slang").unwrap();
        assert!(result.spirv.is_some());
    }
    
    #[test]
    fn test_cache_behavior() {
        let cache = ShaderCache::new();
        let shader = CompiledShader { /* ... */ };
        let key = cache.compute_key("source", "options");
        
        cache.insert(key, shader.clone());
        assert!(cache.get(&key).is_some());
    }
    
    #[test]
    fn test_language_detection() {
        assert_eq!(detect_language("shader.wgsl"), ShaderLanguage::Wgsl);
        assert_eq!(detect_language("shader.slang"), ShaderLanguage::Slang);
    }
}
```

Run tests:
```bash
cargo test --workspace --all-features
# If failing: Check Slang SDK installation, fix compilation errors
```

### Level 3: Integration Test
```bash
# Run the example with new shader system
cargo run --example scene_demo

# Test hot-reload by modifying a shader file
echo "// Modified" >> assets/shaders/basic.slang
# Watch logs for: "Shader hot-reload triggered"

# Expected: Scene renders correctly, hot-reload works
```

### Level 4: Performance Validation
```bash
# Run with release build to test performance
cargo build --release
cargo run --release --example scene_demo

# Check logs for shader compilation times
# Expected: First load <100ms, cached loads <1ms
```

## Final Validation Checklist
- [ ] All tests pass: `cargo test --workspace --all-features`
- [ ] No clippy warnings: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] Formatting correct: `cargo fmt --all -- --check`
- [ ] Documentation builds: `cargo doc --workspace --no-deps`
- [ ] Example runs: `cargo run --example scene_demo`
- [ ] Hot-reload works: Modify shader file while running
- [ ] WGSL shaders still work: Existing shader renders correctly
- [ ] Slang shaders compile: New .slang files work
- [ ] Error handling works: Invalid shaders show helpful errors
- [ ] Performance acceptable: Cached shaders load quickly
- [ ] Logs use tracing: No println! statements

---

## Anti-Patterns to Avoid
- ❌ Don't break existing WGSL shader functionality
- ❌ Don't use println! - always use tracing macros
- ❌ Don't ignore Slang compilation errors - surface them clearly
- ❌ Don't compile shaders on render thread - use async if needed
- ❌ Don't forget to debounce hot-reload events
- ❌ Don't hardcode paths - use the paths utility module
- ❌ Don't skip caching - shader compilation is expensive
- ❌ Don't create new error types - use Box<dyn Error>
- ❌ Don't forget cleanup in drop implementations

## Confidence Score: 8/10

The implementation path is clear with good reference patterns in the codebase. The main complexity is in the Slang compiler integration and ensuring proper SPIR-V generation for WebGPU. The existing hot-reload and caching patterns provide solid foundations to build upon.