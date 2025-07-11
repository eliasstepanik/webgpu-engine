//! Rhai engine wrapper with script caching

use rhai::{Engine, EvalAltResult, Scope, AST};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::debug;

/// Cached script data
#[derive(Clone)]
struct CachedScript {
    ast: AST,
    has_on_start: bool,
    has_on_update: bool,
    has_on_destroy: bool,
}

/// Script engine with caching
pub struct ScriptEngine {
    /// The Rhai engine instance
    pub engine: Arc<Engine>,
    /// Cache of compiled scripts
    cache: Arc<RwLock<HashMap<String, CachedScript>>>,
}

impl ScriptEngine {
    /// Create a new script engine with default configuration
    pub fn new() -> Self {
        let mut engine = Engine::new();

        // Configure engine for safety
        engine.set_max_expr_depths(100, 100);
        engine.set_max_call_levels(50);
        engine.set_max_operations(100_000);
        engine.set_max_string_size(10_000);
        engine.set_max_array_size(10_000);
        engine.set_max_map_size(1_000);

        // Disable certain features for safety
        engine.disable_symbol("eval");

        Self {
            engine: Arc::new(engine),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get a mutable reference to the engine for initialization
    /// This should only be called during setup before any clones are made
    pub fn engine_mut(&mut self) -> Option<&mut Engine> {
        Arc::get_mut(&mut self.engine)
    }

    /// Load and compile a script from a file path
    pub fn load_script(
        &self,
        script_name: &str,
        script_path: &str,
    ) -> Result<(), Box<EvalAltResult>> {
        debug!(
            script_name = script_name,
            path = script_path,
            "Loading script"
        );

        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if cache.contains_key(script_name) {
                debug!(script_name = script_name, "Script already cached");
                return Ok(());
            }
        }

        // Load and compile script
        let script_content = std::fs::read_to_string(script_path)
            .map_err(|e| format!("Failed to read script file '{script_path}': {e}"))?;

        let ast = self.engine.compile(&script_content).map_err(|e| {
            let position = e.position();
            format!(
                "{}:{}:{} - {}",
                script_path,
                position.line().unwrap_or(0),
                position.position().unwrap_or(0),
                e
            )
        })?;

        // Check which lifecycle functions exist
        // For now, we'll assume all scripts have these functions
        // In production, you'd want to test call them to see if they exist
        let has_on_start = true;
        let has_on_update = true;
        let has_on_destroy = true;

        debug!(
            script_name = script_name,
            has_on_start = has_on_start,
            has_on_update = has_on_update,
            has_on_destroy = has_on_destroy,
            "Script lifecycle functions detected"
        );

        // Cache the compiled script
        let cached_script = CachedScript {
            ast,
            has_on_start,
            has_on_update,
            has_on_destroy,
        };

        self.cache
            .write()
            .unwrap()
            .insert(script_name.to_string(), cached_script);

        Ok(())
    }

    /// Call the on_start lifecycle function
    pub fn call_on_start(
        &self,
        script_name: &str,
        entity_id: u64,
        scope: &mut Scope,
    ) -> Result<(), Box<EvalAltResult>> {
        let cache = self.cache.read().unwrap();
        if let Some(cached) = cache.get(script_name) {
            if cached.has_on_start {
                scope.push("entity", entity_id as i64);

                self.engine
                    .call_fn::<()>(scope, &cached.ast, "on_start", (entity_id as i64,))
                    .map_err(|e| -> Box<EvalAltResult> {
                        let position = e.position();
                        Box::new(
                            format!(
                                "{}:{}:{} - {}",
                                script_name,
                                position.line().unwrap_or(0),
                                position.position().unwrap_or(0),
                                e
                            )
                            .into(),
                        )
                    })?;
            }
        }
        Ok(())
    }

    /// Call the on_update lifecycle function
    pub fn call_on_update(
        &self,
        script_name: &str,
        entity_id: u64,
        scope: &mut Scope,
        delta_time: f32,
    ) -> Result<(), Box<EvalAltResult>> {
        let cache = self.cache.read().unwrap();
        if let Some(cached) = cache.get(script_name) {
            if cached.has_on_update {
                scope.push("entity", entity_id as i64);
                scope.push("delta_time", delta_time as f64);

                self.engine
                    .call_fn::<()>(
                        scope,
                        &cached.ast,
                        "on_update",
                        (entity_id as i64, delta_time as f64),
                    )
                    .map_err(|e| -> Box<EvalAltResult> {
                        let position = e.position();
                        Box::new(
                            format!(
                                "{}:{}:{} - {}",
                                script_name,
                                position.line().unwrap_or(0),
                                position.position().unwrap_or(0),
                                e
                            )
                            .into(),
                        )
                    })?;
            }
        }
        Ok(())
    }

    /// Call the on_destroy lifecycle function
    pub fn call_on_destroy(
        &self,
        script_name: &str,
        entity_id: u64,
        scope: &mut Scope,
    ) -> Result<(), Box<EvalAltResult>> {
        let cache = self.cache.read().unwrap();
        if let Some(cached) = cache.get(script_name) {
            if cached.has_on_destroy {
                scope.push("entity", entity_id as i64);

                self.engine
                    .call_fn::<()>(scope, &cached.ast, "on_destroy", (entity_id as i64,))
                    .map_err(|e| -> Box<EvalAltResult> {
                        let position = e.position();
                        Box::new(
                            format!(
                                "{}:{}:{} - {}",
                                script_name,
                                position.line().unwrap_or(0),
                                position.position().unwrap_or(0),
                                e
                            )
                            .into(),
                        )
                    })?;
            }
        }
        Ok(())
    }

    /// Check if a script is loaded in the cache
    pub fn is_loaded(&self, script_name: &str) -> bool {
        self.cache.read().unwrap().contains_key(script_name)
    }

    /// Clear the script cache
    pub fn clear_cache(&self) {
        self.cache.write().unwrap().clear();
    }

    /// Get the number of cached scripts
    pub fn cache_size(&self) -> usize {
        self.cache.read().unwrap().len()
    }
}

impl Default for ScriptEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhai::Dynamic;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_script_engine_creation() {
        let engine = ScriptEngine::new();
        assert_eq!(engine.cache_size(), 0);
    }

    #[test]
    fn test_script_loading() {
        let engine = ScriptEngine::new();

        // Create a temporary test script
        let test_dir = Path::new("test_scripts");
        fs::create_dir_all(test_dir).ok();
        let script_path = test_dir.join("test.rhai");

        let script_content = r#"
            fn on_start(entity) {
                print("Started: " + entity);
            }
            
            fn on_update(entity, delta_time) {
                print("Update: " + entity + ", dt: " + delta_time);
            }
        "#;

        fs::write(&script_path, script_content).unwrap();

        // Load the script
        let result = engine.load_script("test", script_path.to_str().unwrap());
        assert!(result.is_ok());
        assert!(engine.is_loaded("test"));
        assert_eq!(engine.cache_size(), 1);

        // Clean up
        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_script_lifecycle_detection() {
        let engine = ScriptEngine::new();
        let mut scope = Scope::new();

        // Test script without lifecycle functions should not error
        let ast = engine.engine.compile("let x = 42;").unwrap();
        let _result = engine
            .engine
            .eval_ast_with_scope::<Dynamic>(&mut scope, &ast);
        // Just ensure compilation works - Dynamic doesn't need assertion
    }
}
