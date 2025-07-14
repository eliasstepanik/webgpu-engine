## FEATURE:
Add Unity-style script parameters that are editable in the editor inspector

## EXAMPLES:
.claude/examples/console-output-before.txt – Shows current script system output
examples/scene_demo.rs – Demonstrates scene serialization without script parameters

## DOCUMENTATION:
https://docs.unity3d.com/ScriptReference/SerializeField.html
https://rhai.rs/book/engine/metadata/index.html
https://fyrox.rs/blog/post/feature-highlights-0-27/
https://bevy.org/

## OTHER CONSIDERATIONS:
- Current ScriptRef component only stores script name, no parameters
- Scripts hardcode their parameters (e.g., rotation_speed = 1.0)
- Inspector has manual UI code for each component type
- No reflection/property system for automatic UI generation
- Scene serialization uses serde_json::Value for components
- Rhai supports metadata but engine doesn't expose it
- Need to maintain compatibility with existing scripts
- Consider using serde for parameter serialization
- Fyrox uses compile-time reflection for editor properties