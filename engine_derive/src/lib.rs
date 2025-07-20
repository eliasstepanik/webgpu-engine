use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive macro for Component trait
#[proc_macro_derive(Component, attributes(component))]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Extract component name from attribute or use struct name
    let mut component_name = name.to_string();
    for attr in &input.attrs {
        if attr.path().is_ident("component") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    if let Ok(value) = meta.value() {
                        if let Ok(s) = value.parse::<syn::LitStr>() {
                            component_name = s.value();
                        }
                    }
                }
                Ok(())
            });
        }
    }

    // Generate the implementation
    let expanded = quote! {
        impl Component for #name {
            fn component_name() -> &'static str {
                #component_name
            }

            fn register(registry: &mut ComponentRegistry) {
                let metadata = ComponentMetadata::new::<Self>(Self::component_name());
                registry.register_with_metadata(metadata);
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for EditorUI trait
#[proc_macro_derive(EditorUI, attributes(editor))]
pub fn derive_editor_ui(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // For now, just generate a simple implementation
    // The actual UI generation will be handled by the editor
    let expanded = quote! {
        impl EditorUI for #name {
            fn build_ui(
                _component: &mut Self,
                _ui: &mut dyn std::any::Any,
                _entity: hecs::Entity,
            ) -> bool {
                // UI generation is handled by the editor
                // This returns false to indicate no changes were made
                false
            }
        }
    };

    TokenStream::from(expanded)
}