use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

mod field_access_generator;
mod ui_attributes;
mod ui_generator;

use field_access_generator::{generate_field_access_impl, generate_field_access_impl_tuple};
use ui_attributes::{determine_widget_type, parse_ui_attributes};
use ui_generator::generate_ui_metadata_builder;

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

    // For now, hardcode the list of components that have EditorUI
    // This is a workaround because we can't reliably detect EditorUI in the same derive macro expansion
    let components_with_ui = [
        "Transform",
        "Name",
        "Camera",
        "Material",
        "MeshId",
        "GlobalTransform",
        "GlobalWorldTransform",
        "WorldTransform",
        "AABB",
        "Visibility",
        "AudioSource",
        "AudioListener",
        "AmbientSound",
        "AudioMaterial",
    ];
    let has_editor_ui = components_with_ui.contains(&component_name.as_str());

    let metadata_constructor = if has_editor_ui {
        quote! { ComponentMetadata::new_with_ui::<Self>(Self::component_name()) }
    } else {
        quote! { ComponentMetadata::new::<Self>(Self::component_name()) }
    };

    let expanded = quote! {
        impl Component for #name {
            fn component_name() -> &'static str {
                #component_name
            }

            fn register(registry: &mut ComponentRegistry) {
                let metadata = #metadata_constructor;
                registry.register_with_metadata(metadata);
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for EditorUI trait
#[proc_macro_derive(EditorUI, attributes(ui))]
pub fn derive_editor_ui(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Extract fields from the struct
    let (fields, _is_tuple_struct) = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => (Some(fields.named.iter().collect::<Vec<_>>()), false),
            Fields::Unnamed(fields) => (Some(fields.unnamed.iter().collect::<Vec<_>>()), true),
            Fields::Unit => (None, false),
        },
        _ => {
            return TokenStream::from(quote! {
                compile_error!("EditorUI can only be derived for structs");
            });
        }
    };

    // If there are no fields, generate a default implementation
    if fields.is_none() {
        return TokenStream::from(quote! {
            impl EditorUI for #name {
                fn build_ui(
                    _component: &mut Self,
                    _ui: &mut dyn std::any::Any,
                    _entity: hecs::Entity,
                ) -> bool {
                    // No UI for unit structs
                    false
                }
            }
        });
    }

    let fields = fields.unwrap();

    // Parse UI attributes and determine widgets for each field
    let mut field_info = Vec::new();
    for field in fields {
        let attrs = parse_ui_attributes(field);
        let widget = determine_widget_type(&field.ty, &attrs);
        field_info.push((field, attrs, widget));
    }

    // Generate the UI metadata builder
    let ui_metadata_fn = generate_ui_metadata_builder(&field_info);

    // Generate the FieldAccess implementation
    let field_access_impl = if _is_tuple_struct {
        generate_field_access_impl_tuple(name, &field_info)
    } else {
        generate_field_access_impl(name, &field_info)
    };

    // Generate the implementation
    let expanded = quote! {
        impl EditorUI for #name {
            fn build_ui(
                _component: &mut Self,
                _ui: &mut dyn std::any::Any,
                _entity: hecs::Entity,
            ) -> bool {
                // The editor should use the metadata to render UI
                // This default implementation just returns false
                false
            }

            fn ui_metadata() -> Option<crate::component_system::ui_metadata::ComponentUIMetadata> {
                Some(Self::__build_ui_metadata())
            }
        }

        impl #name {
            /// Build UI metadata for this component
            #[doc(hidden)]
            pub fn __build_ui_metadata() -> crate::component_system::ui_metadata::ComponentUIMetadata {
                #ui_metadata_fn
            }
        }

        #field_access_impl
    };

    TokenStream::from(expanded)
}
