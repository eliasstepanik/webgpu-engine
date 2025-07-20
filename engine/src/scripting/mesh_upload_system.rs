//! System for processing mesh uploads from scripts
//!
//! This system processes pending mesh uploads from the script mesh registry
//! and uploads them to the renderer.

use crate::graphics::Renderer;
use crate::scripting::ScriptEngine;
use tracing::debug;

/// Process pending mesh uploads from scripts
pub fn process_script_mesh_uploads(script_engine: &ScriptEngine, renderer: &mut Renderer) {
    let pending_meshes = script_engine.mesh_registry.take_pending_meshes();

    if pending_meshes.is_empty() {
        return;
    }

    debug!(
        count = pending_meshes.len(),
        "Processing pending mesh uploads"
    );

    for pending in pending_meshes {
        debug!(
            name = pending.name,
            callback_id = pending.callback_id,
            "Uploading mesh to renderer"
        );

        // Upload the mesh to the renderer
        let mesh_id = renderer.upload_mesh(&pending.mesh, &pending.name);

        debug!(
            name = pending.name,
            callback_id = pending.callback_id,
            mesh_id = ?mesh_id,
            "Mesh uploaded successfully"
        );

        // Register the uploaded mesh ID
        script_engine
            .mesh_registry
            .register_uploaded(pending.callback_id, mesh_id);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_mesh_upload_system() {
        // This test would require a full renderer setup
        // For now, we just ensure the function compiles
    }
}
