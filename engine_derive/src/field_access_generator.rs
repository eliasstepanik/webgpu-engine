//! Generate FieldAccess trait implementation for components

use crate::ui_attributes::{UIFieldAttribute, UIWidget};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Field;

/// Generate the FieldAccess trait implementation
pub fn generate_field_access_impl(
    type_name: &syn::Ident,
    fields: &[(&Field, UIFieldAttribute, UIWidget)],
) -> TokenStream {
    let get_field_arms = generate_get_field_arms(fields);
    let set_field_arms = generate_set_field_arms(fields);

    quote! {
        impl crate::component_system::field_access::FieldAccess for #type_name {
            fn get_field(&self, field_name: &str) -> Option<crate::component_system::field_access::FieldValue> {
                use crate::component_system::field_access::FieldValue;
                
                match field_name {
                    #(#get_field_arms)*
                    _ => None,
                }
            }
            
            fn set_field(&mut self, field_name: &str, value: crate::component_system::field_access::FieldValue) -> bool {
                use crate::component_system::field_access::FieldValue;
                
                match field_name {
                    #(#set_field_arms)*
                    _ => false,
                }
            }
        }
    }
}

/// Generate match arms for get_field
fn generate_get_field_arms(fields: &[(&Field, UIFieldAttribute, UIWidget)]) -> Vec<TokenStream> {
    fields
        .iter()
        .filter(|(_, attrs, _)| !attrs.hidden)
        .filter_map(|(field, _, widget)| {
            let field_name = field.ident.as_ref()?;
            let field_name_str = field_name.to_string();

            // Only generate field access for supported types
            let value_expr = match widget {
                UIWidget::DragFloat { .. } => {
                    // Check if the field type is actually f32
                    if let syn::Type::Path(path) = &field.ty {
                        if let Some(ident) = path.path.get_ident() {
                            if ident == "f32" || ident == "f64" {
                                quote! { FieldValue::Float(self.#field_name as f32) }
                            } else {
                                return None;
                            }
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }
                UIWidget::DragInt { .. } => {
                    // Check if the field type is actually an integer
                    if let syn::Type::Path(path) = &field.ty {
                        if let Some(ident) = path.path.get_ident() {
                            let ident_str = ident.to_string();
                            if ident_str.contains("32") || ident_str.contains("16") || 
                               ident_str.contains("8") || ident_str == "isize" || ident_str == "usize" {
                                quote! { FieldValue::Int(self.#field_name as i32) }
                            } else {
                                return None;
                            }
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }
                UIWidget::Checkbox => {
                    quote! { FieldValue::Bool(self.#field_name) }
                }
                UIWidget::InputText { .. } => {
                    quote! { FieldValue::String(self.#field_name.clone()) }
                }
                UIWidget::Vec3Input => {
                    quote! { FieldValue::Vec3(self.#field_name) }
                }
                UIWidget::QuatInput => {
                    quote! { FieldValue::Quat(self.#field_name) }
                }
                UIWidget::ColorEdit { alpha } => {
                    if *alpha {
                        quote! { FieldValue::ColorRGBA(self.#field_name) }
                    } else {
                        quote! { FieldValue::ColorRGB(self.#field_name) }
                    }
                }
                UIWidget::Custom(_) => return None, // Skip custom widgets for now
            };

            Some(quote! {
                #field_name_str => Some(#value_expr),
            })
        })
        .collect()
}

/// Generate match arms for set_field
fn generate_set_field_arms(fields: &[(&Field, UIFieldAttribute, UIWidget)]) -> Vec<TokenStream> {
    fields
        .iter()
        .filter(|(_, attrs, _)| !attrs.hidden && !attrs.readonly)
        .filter_map(|(field, _, widget)| {
            let field_name = field.ident.as_ref()?;
            let field_name_str = field_name.to_string();

            // Only generate field access for supported types
            let value_match = match widget {
                UIWidget::DragFloat { .. } => {
                    // Check if the field type is actually f32/f64
                    if let syn::Type::Path(path) = &field.ty {
                        if let Some(ident) = path.path.get_ident() {
                            if ident == "f32" {
                                quote! {
                                    if let Some(v) = value.as_f32() {
                                        self.#field_name = v;
                                        true
                                    } else {
                                        false
                                    }
                                }
                            } else if ident == "f64" {
                                quote! {
                                    if let Some(v) = value.as_f32() {
                                        self.#field_name = v as f64;
                                        true
                                    } else {
                                        false
                                    }
                                }
                            } else {
                                return None;
                            }
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }
                UIWidget::DragInt { .. } => {
                    // Check if the field type is actually an integer
                    if let syn::Type::Path(path) = &field.ty {
                        if let Some(ident) = path.path.get_ident() {
                            let ident_str = ident.to_string();
                            if ident_str == "i32" {
                                quote! {
                                    if let Some(v) = value.as_i32() {
                                        self.#field_name = v;
                                        true
                                    } else {
                                        false
                                    }
                                }
                            } else if ident_str.contains("64") || ident_str.contains("16") || 
                                      ident_str.contains("8") || ident_str == "isize" || ident_str == "usize" {
                                quote! {
                                    if let Some(v) = value.as_i32() {
                                        self.#field_name = v as #ident;
                                        true
                                    } else {
                                        false
                                    }
                                }
                            } else {
                                return None;
                            }
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }
                UIWidget::Checkbox => {
                    quote! {
                        if let Some(v) = value.as_bool() {
                            self.#field_name = v;
                            true
                        } else {
                            false
                        }
                    }
                }
                UIWidget::InputText { .. } => {
                    quote! {
                        if let Some(v) = value.as_string() {
                            self.#field_name = v.clone();
                            true
                        } else {
                            false
                        }
                    }
                }
                UIWidget::Vec3Input => {
                    quote! {
                        if let Some(v) = value.as_vec3() {
                            self.#field_name = v;
                            true
                        } else {
                            false
                        }
                    }
                }
                UIWidget::QuatInput => {
                    quote! {
                        if let Some(v) = value.as_quat() {
                            self.#field_name = v;
                            true
                        } else {
                            false
                        }
                    }
                }
                UIWidget::ColorEdit { alpha } => {
                    if *alpha {
                        quote! {
                            if let Some(v) = value.as_color_rgba() {
                                self.#field_name = v;
                                true
                            } else {
                                false
                            }
                        }
                    } else {
                        quote! {
                            if let Some(v) = value.as_color_rgb() {
                                self.#field_name = v;
                                true
                            } else {
                                false
                            }
                        }
                    }
                }
                UIWidget::Custom(_) => return None, // Skip custom widgets for now
            };

            Some(quote! {
                #field_name_str => #value_match,
            })
        })
        .collect()
}

/// Generate FieldAccess for tuple structs
pub fn generate_field_access_impl_tuple(
    type_name: &syn::Ident,
    fields: &[(&Field, UIFieldAttribute, UIWidget)],
) -> TokenStream {
    // For tuple structs with a single field, we can provide limited support
    if fields.len() == 1 {
        let (_, attrs, widget) = &fields[0];
        
        if attrs.hidden {
            return generate_empty_field_access(type_name);
        }
        
        let get_value = match widget {
            UIWidget::InputText { .. } => quote! { FieldValue::String(self.0.clone()) },
            _ => return generate_empty_field_access(type_name),
        };
        
        let set_value = if !attrs.readonly {
            match widget {
                UIWidget::InputText { .. } => quote! {
                    if let Some(v) = value.as_string() {
                        self.0 = v.clone();
                        true
                    } else {
                        false
                    }
                },
                _ => quote! { false },
            }
        } else {
            quote! { false }
        };
        
        quote! {
            impl crate::component_system::field_access::FieldAccess for #type_name {
                fn get_field(&self, field_name: &str) -> Option<crate::component_system::field_access::FieldValue> {
                    use crate::component_system::field_access::FieldValue;
                    
                    match field_name {
                        "0" => Some(#get_value),
                        _ => None,
                    }
                }
                
                fn set_field(&mut self, field_name: &str, value: crate::component_system::field_access::FieldValue) -> bool {
                    use crate::component_system::field_access::FieldValue;
                    
                    match field_name {
                        "0" => #set_value,
                        _ => false,
                    }
                }
            }
        }
    } else {
        generate_empty_field_access(type_name)
    }
}

/// Generate empty FieldAccess implementation
fn generate_empty_field_access(type_name: &syn::Ident) -> TokenStream {
    quote! {
        impl crate::component_system::field_access::FieldAccess for #type_name {
            fn get_field(&self, _field_name: &str) -> Option<crate::component_system::field_access::FieldValue> {
                None
            }
            
            fn set_field(&mut self, _field_name: &str, _value: crate::component_system::field_access::FieldValue) -> bool {
                false
            }
        }
    }
}