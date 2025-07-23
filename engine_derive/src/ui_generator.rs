//! UI metadata generation for the EditorUI derive macro

use crate::ui_attributes::{UIFieldAttribute, UIWidget};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Field;

/// Generate the UI metadata builder function
pub fn generate_ui_metadata_builder(
    fields: &[(&Field, UIFieldAttribute, UIWidget)],
) -> TokenStream {
    let field_metadata: Vec<TokenStream> = fields
        .iter()
        .filter(|(_, attrs, _)| !attrs.hidden)
        .enumerate()
        .map(|(index, (field, attrs, widget))| generate_field_metadata(field, attrs, widget, index))
        .collect();

    quote! {
        let mut metadata = crate::component_system::ui_metadata::ComponentUIMetadata::new();
        #(#field_metadata)*
        metadata
    }
}

/// Generate metadata for a single field
fn generate_field_metadata(
    field: &Field,
    attrs: &UIFieldAttribute,
    widget: &UIWidget,
    index: usize,
) -> TokenStream {
    let (field_name_str, default_label) = if let Some(ident) = &field.ident {
        (ident.to_string(), ident.to_string())
    } else {
        // For tuple structs, use index
        (format!("{index}"), format!("Field {index}"))
    };
    let label = attrs.label.as_ref().unwrap_or(&default_label);

    let widget_metadata = generate_widget_metadata(widget);
    let tooltip = match &attrs.tooltip {
        Some(t) => quote! { Some(#t.to_string()) },
        None => quote! { None },
    };
    let readonly = attrs.readonly;

    quote! {
        {
            let field_metadata = crate::component_system::ui_metadata::UIFieldMetadata {
                name: #field_name_str.to_string(),
                label: Some(#label.to_string()),
                widget: #widget_metadata,
                tooltip: #tooltip,
                hidden: false,
                readonly: #readonly,
                properties: std::collections::HashMap::new(),
            };
            metadata.add_field(field_metadata);
        }
    }
}

/// Generate widget metadata based on widget type
fn generate_widget_metadata(widget: &UIWidget) -> TokenStream {
    use UIWidget::*;

    match widget {
        DragFloat { min, max, speed } => quote! {
            crate::component_system::ui_metadata::UIWidgetType::DragFloat {
                min: #min,
                max: #max,
                speed: #speed,
                format: "%.3f".to_string(),
            }
        },

        DragInt { min, max, speed } => quote! {
            crate::component_system::ui_metadata::UIWidgetType::DragInt {
                min: #min,
                max: #max,
                speed: #speed,
                format: "%d".to_string(),
            }
        },

        InputText { multiline, .. } => quote! {
            crate::component_system::ui_metadata::UIWidgetType::InputText {
                multiline: #multiline,
                max_length: None,
            }
        },

        Checkbox => quote! {
            crate::component_system::ui_metadata::UIWidgetType::Checkbox
        },

        ColorEdit { alpha } => quote! {
            crate::component_system::ui_metadata::UIWidgetType::ColorEdit {
                alpha: #alpha,
            }
        },

        Vec3Input => quote! {
            crate::component_system::ui_metadata::UIWidgetType::Vec3Input {
                speed: 0.01,
                format: "%.3f".to_string(),
            }
        },

        QuatInput => quote! {
            crate::component_system::ui_metadata::UIWidgetType::QuatInput {
                speed: 0.5,
                format: "%.1f".to_string(),
            }
        },

        Custom(path) => {
            let path_str = path
                .segments
                .iter()
                .map(|seg| seg.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            quote! {
                crate::component_system::ui_metadata::UIWidgetType::Custom {
                    function: #path_str.to_string(),
                }
            }
        }
    }
}
