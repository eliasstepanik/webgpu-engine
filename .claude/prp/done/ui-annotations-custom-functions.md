name: "UI Annotations and Custom UI Functions Implementation"
description: |

## Purpose
Implement a comprehensive UI annotation system for components using derive macros to reduce boilerplate, improve consistency, and make component UI creation more declarative.

## Core Principles
1. **Context is King**: Include ALL necessary documentation, examples, and caveats
2. **Validation Loops**: Provide executable tests/lints the AI can run and fix
3. **Information Dense**: Use keywords and patterns from the codebase
4. **Progressive Success**: Start simple, validate, then enhance
5. **Global rules**: Be sure to follow all rules in CLAUDE.md

---

## Goal
Create a derive macro system that automatically generates UI for components based on field types and custom attributes, eliminating the need for manual UI code in the inspector panel while providing flexibility for custom UI implementations.

## Why
- **Reduce Boilerplate**: Currently every component needs manual UI code in inspector.rs
- **Improve Consistency**: Ensure all components follow the same UI patterns
- **Developer Experience**: Make it trivial to add UI to new components
- **Maintainability**: Changes to UI behavior can be made in one place
- **Flexibility**: Support both automatic and custom UI generation

## What
Extend the existing `#[derive(EditorUI)]` macro to:
1. Parse UI attributes on struct fields
2. Generate appropriate imgui widgets based on field types
3. Support common UI customizations (ranges, tooltips, etc.)
4. Allow custom UI functions for complex cases
5. Provide sensible defaults for common types (Vec3, Quat, bool, String, etc.)

### Success Criteria
- [ ] All built-in components use UI annotations instead of manual UI code
- [ ] New components automatically get appropriate UI without additional code
- [ ] Custom UI functions can be specified for complex cases
- [ ] Performance is equal or better than manual implementation
- [ ] Macro provides helpful error messages for invalid attributes

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://docs.rs/syn/latest/syn/
  why: Parse attributes and generate code with syn crate
  
- url: https://docs.rs/quote/latest/quote/
  why: Generate Rust code in procedural macros
  
- url: https://docs.rs/imgui/latest/imgui/
  why: imgui-rs widget API reference for UI generation
  
- url: https://github.com/jakobhellermann/bevy-inspector-egui
  why: Reference implementation of similar system in Bevy
  section: Derive macro implementation and attribute parsing

- file: engine_derive/src/lib.rs
  why: Current derive macro implementation to extend
  
- file: engine/src/component_system/mod.rs
  why: Component and EditorUI traits, UIBuilderFn type definition
  
- file: editor/src/panels/inspector.rs
  why: Current manual UI implementations to replicate
  
- file: engine/src/scripting/property_parser.rs
  why: Reference for existing annotation parsing patterns
```

### Current Codebase Structure
```bash
engine/
├── src/
│   ├── component_system/
│   │   └── mod.rs          # Component, EditorUI traits, UIBuilderFn
│   ├── core/
│   │   └── entity/
│   │       └── components.rs  # Built-in components
│   └── scripting/
│       └── property_parser.rs # Reference annotation parser
editor/
├── src/
│   └── panels/
│       └── inspector.rs    # Manual UI implementations
engine_derive/
├── src/
│   └── lib.rs             # Derive macro implementations
```

### Desired Changes
```bash
engine_derive/
├── src/
│   ├── lib.rs             # Extended with UI attribute parsing
│   ├── ui_attributes.rs   # New: UI attribute parsing logic
│   └── ui_generator.rs    # New: Code generation for UI
engine/
├── src/
│   └── component_system/
│       └── ui_defaults.rs # New: Default UI implementations
```

### Known Gotchas & Library Quirks
```rust
// CRITICAL: imgui::Ui is passed as &mut dyn Any in UIBuilderFn
// Must cast back: let ui = imgui_any.downcast_mut::<imgui::Ui>().unwrap();

// CRITICAL: World access in UI builders is mutable - be careful with entity mutations
// Only mutate the specific component being edited

// GOTCHA: imgui is immediate mode - widgets return true when value changes
// Use this to track modifications: if ui.drag_float(...) { return true; }

// PATTERN: Always use ui.set_next_item_width(-1.0) for full-width inputs
// PATTERN: Use ui.same_line() for horizontal layout of Vec3 components
```

## Implementation Blueprint

### Data Models and Structure

```rust
// UI attribute structure in engine_derive/src/ui_attributes.rs
#[derive(Debug)]
struct UIFieldAttribute {
    // Numeric constraints
    range: Option<(f32, f32)>,      // min, max values
    step: Option<f32>,              // drag step size
    speed: Option<f32>,             // drag speed
    
    // Display options
    tooltip: Option<String>,        // hover tooltip
    label: Option<String>,          // custom label
    format: Option<String>,         // printf-style format
    
    // Behavior flags
    hidden: bool,                   // skip in UI
    readonly: bool,                 // display only
    multiline: Option<u32>,         // text lines
    
    // Custom UI
    custom: Option<syn::Path>,      // custom UI function
    color_mode: Option<String>,     // "rgb" or "rgba"
}

// Type to UI widget mapping
enum UIWidget {
    DragFloat { min: f32, max: f32, speed: f32 },
    DragInt { min: i32, max: i32, speed: f32 },
    InputText { multiline: bool, hint: Option<String> },
    Checkbox,
    ColorEdit { alpha: bool },
    Vec3Input,
    QuatInput,
    Custom(syn::Path),
}
```

### List of Tasks

```yaml
Task 1: Extend derive macro to parse UI attributes
MODIFY engine_derive/src/lib.rs:
  - FIND pattern: "pub fn derive_editor_ui"
  - ADD UI attribute parsing before quote!
  - PARSE field attributes looking for #[ui(...)]

CREATE engine_derive/src/ui_attributes.rs:
  - IMPLEMENT attribute parsing using syn
  - SUPPORT all UI attribute options
  - PROVIDE clear error messages for invalid attributes

Task 2: Create UI widget generators
CREATE engine_derive/src/ui_generator.rs:
  - IMPLEMENT type to widget mapping
  - GENERATE imgui code for each widget type
  - HANDLE nested types (Option<T>, Vec<T>)

Task 3: Generate EditorUI implementations
MODIFY engine_derive/src/lib.rs:
  - GENERATE EditorUI::ui_builder implementation
  - CREATE UIBuilderFn closure with proper type casting
  - EMIT code for each field based on type and attributes

Task 4: Create default UI implementations
CREATE engine/src/component_system/ui_defaults.rs:
  - IMPLEMENT default UI for Vec3 (3 drag inputs)
  - IMPLEMENT default UI for Quat (euler angles)
  - IMPLEMENT default UI for arrays/vecs
  - EXPORT for use in generated code

Task 5: Update existing components
MODIFY engine/src/core/entity/components.rs:
  - ADD UI attributes to Transform, Camera, Material
  - REMOVE manual UI code from inspector.rs
  - TEST each component renders correctly

Task 6: Add comprehensive tests
CREATE engine_derive/tests/ui_macro_tests.rs:
  - TEST macro expansion for various types
  - TEST error cases (invalid attributes)
  - TEST custom UI function references

Task 7: Update documentation
MODIFY CLAUDE.md:
  - ADD UI annotation guidelines
  - PROVIDE examples for each attribute
  - DOCUMENT custom UI function API
```

### Per Task Pseudocode

```rust
// Task 1: Parse UI attributes
fn parse_ui_attributes(field: &syn::Field) -> UIFieldAttribute {
    let mut attrs = UIFieldAttribute::default();
    
    for attr in &field.attrs {
        if attr.path.is_ident("ui") {
            // Parse attribute content: #[ui(range = 0.0..1.0, tooltip = "...")]
            attr.parse_args_with(|input: ParseStream| {
                // Parse key = value pairs
                while !input.is_empty() {
                    let key: Ident = input.parse()?;
                    input.parse::<Token![=]>()?;
                    
                    match key.to_string().as_str() {
                        "range" => {
                            // Parse range expression: 0.0..1.0
                            let min: f32 = input.parse()?;
                            input.parse::<Token![..]>()?;
                            let max: f32 = input.parse()?;
                            attrs.range = Some((min, max));
                        }
                        "tooltip" => {
                            let tooltip: LitStr = input.parse()?;
                            attrs.tooltip = Some(tooltip.value());
                        }
                        // ... handle other attributes
                    }
                }
            });
        }
    }
    attrs
}

// Task 3: Generate UI code
fn generate_field_ui(field: &Field, attrs: &UIFieldAttribute) -> TokenStream {
    let field_name = &field.ident;
    let field_str = field_name.to_string();
    
    // Determine widget based on type
    let widget_code = match &field.ty {
        Type::Path(path) if is_f32(&path) => {
            let (min, max) = attrs.range.unwrap_or((-f32::MAX, f32::MAX));
            let speed = attrs.speed.unwrap_or(0.01);
            quote! {
                if imgui::Drag::new(concat!(#field_str, "##", #field_str))
                    .range(#min..=#max)
                    .speed(#speed)
                    .build(ui, &mut component.#field_name)
                {
                    changed = true;
                }
            }
        }
        Type::Path(path) if is_bool(&path) => {
            quote! {
                if ui.checkbox(#field_str, &mut component.#field_name) {
                    changed = true;
                }
            }
        }
        // ... handle other types
    };
    
    // Add tooltip if specified
    if let Some(tooltip) = &attrs.tooltip {
        quote! {
            #widget_code
            if ui.is_item_hovered() {
                ui.tooltip_text(#tooltip);
            }
        }
    } else {
        widget_code
    }
}
```

### Integration Points
```yaml
DERIVE MACRO:
  - location: engine_derive/src/lib.rs
  - pattern: Use existing Component derive as reference
  
COMPONENT SYSTEM:
  - import: engine/src/component_system/ui_defaults.rs
  - usage: Generated code calls default UI functions
  
INSPECTOR:
  - remove: Manual UI implementations in inspector.rs
  - replace: Use component metadata UI builders
```

## Validation Loop

### Level 1: Syntax & Style
```bash
# Check macro crate
cd engine_derive && cargo check
cd .. && cargo clippy --workspace --all-targets --all-features -- -D warnings

# Expected: No errors or warnings
```

### Level 2: Macro Expansion Tests
```rust
// CREATE engine_derive/tests/ui_macro_tests.rs
#[test]
fn test_simple_component_ui() {
    let input = quote! {
        #[derive(Component, EditorUI)]
        struct TestComponent {
            #[ui(range = 0.0..1.0, tooltip = "Test value")]
            value: f32,
            
            #[ui(hidden)]
            internal: u32,
        }
    };
    
    let output = derive_editor_ui(input);
    // Verify generated code contains drag float with range
    assert!(output.to_string().contains("Drag::new"));
    assert!(output.to_string().contains("range(0.0..=1.0)"));
}

#[test]
fn test_vec3_default_ui() {
    // Test that Vec3 generates 3 drag inputs
}

#[test]
fn test_custom_ui_function() {
    // Test custom UI function reference
}
```

```bash
# Run macro tests
cd engine_derive && cargo test
```

### Level 3: Integration Test
```rust
// Add test component to engine
#[derive(Component, EditorUI)]
#[component(name = "UITestComponent")]
pub struct UITestComponent {
    #[ui(range = 0.0..10.0, step = 0.1)]
    pub speed: f32,
    
    #[ui(tooltip = "Component color")]
    pub color: [f32; 4],
    
    pub position: Vec3,
}

// Run editor and verify:
// 1. Speed shows as drag with correct range
// 2. Color shows as color picker
// 3. Position shows as 3 separate inputs
// 4. Tooltips appear on hover
```

```bash
# Build and run editor
just preflight
just run-editor

# Manually test:
# 1. Add UITestComponent to entity
# 2. Verify all fields render correctly
# 3. Verify value changes are saved
```

## Final Validation Checklist
- [ ] All derive macro tests pass: `cd engine_derive && cargo test`
- [ ] No clippy warnings: `cargo clippy --workspace`
- [ ] Editor builds successfully: `cargo build --bin editor`
- [ ] Manual test: Components render with correct UI
- [ ] Manual test: UI attributes work (range, tooltip, etc.)
- [ ] Manual test: Custom UI functions work
- [ ] Performance: UI generation time comparable to manual
- [ ] Documentation: CLAUDE.md updated with examples

---

## Anti-Patterns to Avoid
- ❌ Don't generate UI code at runtime - use compile-time macros
- ❌ Don't mutate World during UI rendering except for the component
- ❌ Don't forget to handle the type-erased imgui::Ui pointer
- ❌ Don't skip validation for custom UI functions
- ❌ Don't hardcode widget parameters - use attributes
- ❌ Don't ignore imgui's immediate mode constraints

## Implementation Notes

1. **Start Simple**: Begin with basic types (f32, bool, String) before complex ones
2. **Type Safety**: Use syn's type parsing to ensure correct widget selection
3. **Error Messages**: Provide clear compile errors for invalid attributes
4. **Performance**: Generated code should be as efficient as hand-written
5. **Compatibility**: Maintain compatibility with existing EditorUI implementations

## Confidence Score: 8/10

The implementation path is clear with good reference examples. Main complexity is in the macro parsing and code generation, but the patterns from existing derive macros and bevy-inspector-egui provide solid guidance.