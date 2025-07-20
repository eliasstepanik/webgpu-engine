//! UI attribute parsing for the EditorUI derive macro

use syn::{
    parse::{Parse, ParseStream},
    Field, LitFloat, LitInt, LitStr, Path, Token,
};

/// Parsed UI field attributes from #[ui(...)]
#[derive(Debug, Default, Clone)]
pub struct UIFieldAttribute {
    /// Numeric constraints: (min, max)
    pub range: Option<(f32, f32)>,
    /// Drag step size
    pub step: Option<f32>,
    /// Drag speed
    pub speed: Option<f32>,

    /// Display options
    pub tooltip: Option<String>,
    pub label: Option<String>,
    pub format: Option<String>,

    /// Behavior flags
    pub hidden: bool,
    pub readonly: bool,
    pub multiline: Option<u32>,

    /// Custom UI
    pub custom: Option<Path>,
    pub color_mode: Option<String>,
}

/// Type of UI widget to generate
#[derive(Debug, Clone)]
pub enum UIWidget {
    DragFloat {
        min: f32,
        max: f32,
        speed: f32,
    },
    DragInt {
        min: i32,
        max: i32,
        speed: f32,
    },
    InputText {
        multiline: bool,
        #[allow(dead_code)]
        hint: Option<String>,
    },
    Checkbox,
    ColorEdit {
        alpha: bool,
    },
    Vec3Input,
    QuatInput,
    Custom(Path),
}

/// Parse UI attributes from a field
pub fn parse_ui_attributes(field: &Field) -> UIFieldAttribute {
    let mut attrs = UIFieldAttribute::default();

    for attr in &field.attrs {
        if attr.path().is_ident("ui") {
            if let Err(e) = attr.parse_nested_meta(|meta| parse_ui_meta(&meta, &mut attrs)) {
                eprintln!("Failed to parse UI attribute: {e}");
            }
        }
    }

    attrs
}

/// Parse a single UI meta item
fn parse_ui_meta(
    meta: &syn::meta::ParseNestedMeta,
    attrs: &mut UIFieldAttribute,
) -> syn::Result<()> {
    let ident = meta
        .path
        .get_ident()
        .ok_or_else(|| syn::Error::new_spanned(&meta.path, "expected identifier"))?
        .to_string();

    match ident.as_str() {
        "range" => {
            let value = meta.value()?;
            let range_expr: RangeExpr = value.parse()?;
            attrs.range = Some((range_expr.min, range_expr.max));
        }
        "step" => {
            let value = meta.value()?;
            let lit: LitFloat = value.parse()?;
            attrs.step = Some(lit.base10_parse()?);
        }
        "speed" => {
            let value = meta.value()?;
            let lit: LitFloat = value.parse()?;
            attrs.speed = Some(lit.base10_parse()?);
        }
        "tooltip" => {
            let value = meta.value()?;
            let lit: LitStr = value.parse()?;
            attrs.tooltip = Some(lit.value());
        }
        "label" => {
            let value = meta.value()?;
            let lit: LitStr = value.parse()?;
            attrs.label = Some(lit.value());
        }
        "format" => {
            let value = meta.value()?;
            let lit: LitStr = value.parse()?;
            attrs.format = Some(lit.value());
        }
        "hidden" => {
            attrs.hidden = true;
        }
        "readonly" => {
            attrs.readonly = true;
        }
        "multiline" => {
            let value = meta.value()?;
            let lit: LitInt = value.parse()?;
            attrs.multiline = Some(lit.base10_parse()?);
        }
        "custom" => {
            let value = meta.value()?;
            let path: Path = value.parse()?;
            attrs.custom = Some(path);
        }
        "color_mode" => {
            let value = meta.value()?;
            let lit: LitStr = value.parse()?;
            attrs.color_mode = Some(lit.value());
        }
        _ => {
            return Err(syn::Error::new_spanned(
                &meta.path,
                format!("unknown UI attribute: {ident}"),
            ));
        }
    }

    Ok(())
}

/// Helper struct to parse range expressions like 0.0..1.0
struct RangeExpr {
    min: f32,
    max: f32,
}

impl Parse for RangeExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse min value
        let min = if let Ok(lit) = input.parse::<LitFloat>() {
            lit.base10_parse()?
        } else if let Ok(lit) = input.parse::<LitInt>() {
            lit.base10_parse::<i32>()? as f32
        } else {
            return Err(input.error("expected numeric literal"));
        };

        // Parse .. or ..=
        let inclusive = if input.peek(Token![..=]) {
            input.parse::<Token![..=]>()?;
            true
        } else if input.peek(Token![..]) {
            input.parse::<Token![..]>()?;
            false
        } else {
            return Err(input.error("expected .. or ..="));
        };

        // Parse max value
        let max = if let Ok(lit) = input.parse::<LitFloat>() {
            lit.base10_parse()?
        } else if let Ok(lit) = input.parse::<LitInt>() {
            lit.base10_parse::<i32>()? as f32
        } else {
            return Err(input.error("expected numeric literal"));
        };

        // Adjust for inclusive range
        if !inclusive {
            // For exclusive ranges, we'll treat them as inclusive in the UI
            // This is a simplification, but works well for UI purposes
        }

        Ok(RangeExpr { min, max })
    }
}

/// Determine the UI widget type based on the field type
pub fn determine_widget_type(field_type: &syn::Type, attrs: &UIFieldAttribute) -> UIWidget {
    // Check for custom UI function first
    if let Some(custom_path) = &attrs.custom {
        return UIWidget::Custom(custom_path.clone());
    }

    // Parse the type and determine widget
    match field_type {
        syn::Type::Path(type_path) => {
            let type_str = type_path
                .path
                .segments
                .last()
                .map(|seg| seg.ident.to_string())
                .unwrap_or_default();

            match type_str.as_str() {
                "f32" | "f64" => {
                    let (min, max) = attrs.range.unwrap_or((-f32::MAX, f32::MAX));
                    let speed = attrs.speed.or(attrs.step).unwrap_or(0.01);
                    UIWidget::DragFloat { min, max, speed }
                }
                "i32" | "i64" | "u32" | "u64" | "i16" | "u16" | "i8" | "u8" | "isize" | "usize" => {
                    let (min, max) = attrs
                        .range
                        .map(|(min, max)| (min as i32, max as i32))
                        .unwrap_or((i32::MIN, i32::MAX));
                    let speed = attrs.speed.or(attrs.step).unwrap_or(1.0);
                    UIWidget::DragInt { min, max, speed }
                }
                "bool" => UIWidget::Checkbox,
                "String" => UIWidget::InputText {
                    multiline: attrs.multiline.is_some(),
                    hint: attrs.tooltip.clone(),
                },
                "Vec3" => UIWidget::Vec3Input,
                "Quat" => UIWidget::QuatInput,
                _ => {
                    // Custom widget for unsupported types
                    UIWidget::Custom(syn::parse_quote! { unsupported })
                }
            }
        }
        syn::Type::Array(array) => {
            // Check if it's a color array [f32; 4]
            if let syn::Type::Path(elem_path) = &*array.elem {
                let elem_str = elem_path
                    .path
                    .segments
                    .last()
                    .map(|seg| seg.ident.to_string())
                    .unwrap_or_default();

                if elem_str == "f32" {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Int(int_lit),
                        ..
                    }) = &array.len
                    {
                        let len: usize = int_lit.base10_parse().unwrap_or(0);
                        if len == 4 || len == 3 {
                            let alpha = len == 4 || attrs.color_mode == Some("rgba".to_string());
                            return UIWidget::ColorEdit { alpha };
                        }
                    }
                }
            }

            // Default array handling - unsupported
            UIWidget::Custom(syn::parse_quote! { unsupported })
        }
        _ => {
            // Default for unknown types - unsupported
            UIWidget::Custom(syn::parse_quote! { unsupported })
        }
    }
}
