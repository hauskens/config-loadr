use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{Attribute, Data, DeriveInput, Fields, Meta, Token, Type, parse_macro_input};

/// Helper enum for parsed attribute values
enum MetaValue {
    Str(String),
    Expr(syn::Expr),
    Flag,
}

/// Check if the struct has #[allow(missing_docs)] attribute
fn check_allow_missing_docs(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("allow") {
            attr.parse_args::<syn::Ident>()
                .map(|ident| ident == "missing_docs")
                .unwrap_or(false)
        } else {
            false
        }
    })
}

/// Main macro for defining configuration structs with automatic loading
#[proc_macro]
pub fn define_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match generate_config(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn generate_config(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;
    let vis = &input.vis;
    let struct_attrs = &input.attrs;

    // Check for struct-level attributes
    let allow_missing_docs = check_allow_missing_docs(struct_attrs);

    // Extract fields from the struct
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "define_config! only supports structs with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "define_config! only supports structs",
            ));
        }
    };

    let mut field_defs = Vec::new();
    let mut load_impl_fields = Vec::new();
    let mut load_impl_unwraps = Vec::new();
    let mut docs_impl_fields = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_vis = &field.vis;
        let field_type = &field.ty;
        let field_attrs = &field.attrs;

        // Parse field configuration from attributes
        let config = parse_field_config(field_attrs, allow_missing_docs)?;

        // Extract cfg attributes for feature gating
        let cfg_attrs: Vec<&Attribute> = field_attrs
            .iter()
            .filter(|attr| attr.path().is_ident("cfg"))
            .collect();

        // Generate the field definition with ConfigField wrapper
        let inner_type = extract_inner_type(field_type);
        field_defs.push(quote! {
            #(#cfg_attrs)*
            #field_vis #field_name: ::config_loadr::ConfigField<#field_type>
        });

        // Generate load implementation code
        let env_var = &config.env_var;
        let description = &config.description;

        // For optional fields, extract the inner type from Option<T>
        let (is_option, actual_type) = extract_option_type(field_type);

        let load_code = match config.mode {
            FieldMode::Required => {
                let example = config.example.as_ref().ok_or_else(|| {
                    syn::Error::new_spanned(
                        field,
                        "required fields must have an #[example] attribute",
                    )
                })?;

                quote! {
                    #(#cfg_attrs)*
                    let #field_name = builder.required::<#inner_type>(
                        #env_var,
                        #description,
                        #example,
                    );
                }
            }
            FieldMode::Default(ref default_expr) => {
                // Skip compile-time validation - it's too restrictive
                // Users should rely on tests instead

                quote! {
                    #(#cfg_attrs)*
                    let #field_name = builder.or_default::<#inner_type>(
                        #env_var,
                        #description,
                        #default_expr,
                    );
                }
            }
            FieldMode::Optional => {
                let example = config.example.as_ref().ok_or_else(|| {
                    syn::Error::new_spanned(
                        field,
                        "optional fields must have an #[example] attribute",
                    )
                })?;

                // For optional fields, use the inner type (without Option wrapper)
                let opt_inner = if is_option {
                    actual_type
                } else {
                    return Err(syn::Error::new_spanned(
                        field,
                        "optional fields must have type Option<T>",
                    ));
                };

                quote! {
                    #(#cfg_attrs)*
                    let #field_name = builder.optional::<#opt_inner>(
                        #env_var,
                        #description,
                        #example,
                    );
                }
            }
        };

        load_impl_fields.push(load_code.clone());

        // For optional fields, we don't unwrap (they're already ConfigField<Option<T>>)
        let unwrap_code = if matches!(config.mode, FieldMode::Optional) {
            quote! {
                #(#cfg_attrs)*
                #field_name
            }
        } else {
            quote! {
                #(#cfg_attrs)*
                #field_name: #field_name.unwrap()
            }
        };
        load_impl_unwraps.push(unwrap_code);

        // Same load code for docs
        docs_impl_fields.push(load_code);
    }

    // Filter out our custom attributes (allow(missing_docs)) from struct definition
    let filtered_attrs: Vec<&Attribute> = struct_attrs
        .iter()
        .filter(|attr| {
            // Keep the attribute unless it's our custom ones
            if attr.path().is_ident("allow") {
                // Check if it's allow(missing_docs)
                if let Ok(ident) = attr.parse_args::<syn::Ident>()
                    && ident == "missing_docs"
                {
                    return false;
                }
            }
            true
        })
        .collect();

    // Generate the struct definition
    let struct_def = quote! {
        #(#filtered_attrs)*
        #vis struct #struct_name {
            #(#field_defs),*
        }
    };

    // Generate Load trait implementation
    let load_impl = quote! {
        impl ::config_loadr::Load for #struct_name {
            fn load() -> Self {
                let _ = dotenvy::dotenv();
                let mut builder = ::config_loadr::ConfigBuilder::new();

                #(#load_impl_fields)*

                builder.finish_or_panic();

                Self {
                    #(#load_impl_unwraps),*
                }
            }

            fn load_or_error() -> Result<Self, Vec<::config_loadr::ConfigError>> {
                let _ = dotenvy::dotenv();
                let mut builder = ::config_loadr::ConfigBuilder::new();

                #(#load_impl_fields)*

                builder.finish()?;

                Ok(Self {
                    #(#load_impl_unwraps),*
                })
            }

            #[allow(unused_variables)]
            fn builder_for_docs() -> ::config_loadr::ConfigBuilder {
                let mut builder = ::config_loadr::ConfigBuilder::new();

                #(#docs_impl_fields)*

                builder
            }
        }
    };

    Ok(quote! {
        #struct_def
        #load_impl
    })
}

#[derive(Debug)]
struct FieldConfig {
    env_var: String,
    description: String,
    example: Option<syn::Expr>,
    mode: FieldMode,
}

#[derive(Debug)]
enum FieldMode {
    Required,
    Default(syn::Expr),
    Optional,
}

/// Parse #[field(env = "X", doc = "Y", default = val)] syntax
fn parse_field_list(meta_list: &syn::MetaList) -> syn::Result<HashMap<String, MetaValue>> {
    let mut values = HashMap::new();

    meta_list.parse_nested_meta(|meta| {
        let key = meta
            .path
            .get_ident()
            .ok_or_else(|| meta.error("expected identifier"))?
            .to_string();

        if meta.input.peek(Token![=]) {
            meta.input.parse::<Token![=]>()?;

            if key == "env" || key == "doc" {
                let value: syn::LitStr = meta.input.parse()?;
                values.insert(key, MetaValue::Str(value.value()));
            } else {
                let expr: syn::Expr = meta.input.parse()?;
                values.insert(key, MetaValue::Expr(expr));
            }
        } else {
            values.insert(key, MetaValue::Flag);
        }

        Ok(())
    })?;

    Ok(values)
}

fn parse_field_config(attrs: &[Attribute], allow_missing_docs: bool) -> syn::Result<FieldConfig> {
    // Find the #[field(...)] attribute
    let field_attr = attrs.iter()
        .find(|attr| attr.path().is_ident("field"))
        .ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "field must have #[field(...)] attribute with env, doc, and mode (required/default/optional)"
            )
        })?;

    // Parse it as a Meta::List
    let parsed = match &field_attr.meta {
        Meta::List(list) => parse_field_list(list)?,
        _ => {
            return Err(syn::Error::new_spanned(
                field_attr,
                "field attribute must be a list: #[field(env = \"...\", ...)]",
            ));
        }
    };

    // Extract env (required)
    let env_var = match parsed.get("env") {
        Some(MetaValue::Str(s)) => s.clone(),
        _ => {
            return Err(syn::Error::new_spanned(
                field_attr,
                "field must have env = \"VAR_NAME\"",
            ));
        }
    };

    // Extract doc (conditionally required)
    let description = match parsed.get("doc") {
        Some(MetaValue::Str(s)) => s.trim().to_string(),
        None if allow_missing_docs => String::new(),
        None => {
            return Err(syn::Error::new_spanned(
                field_attr,
                "field must have doc = \"description\" (or use #[allow(missing_docs)] on struct)",
            ));
        }
        _ => {
            return Err(syn::Error::new_spanned(
                field_attr,
                "doc must be a string literal",
            ));
        }
    };

    // Extract example (optional)
    let example = parsed.get("example").and_then(|v| match v {
        MetaValue::Expr(e) => Some(e.clone()),
        _ => None,
    });

    // Extract mode (required, default, or optional)
    let mode = if parsed.contains_key("required") {
        FieldMode::Required
    } else if let Some(MetaValue::Expr(e)) = parsed.get("default") {
        FieldMode::Default(e.clone())
    } else if parsed.contains_key("optional") {
        FieldMode::Optional
    } else {
        return Err(syn::Error::new_spanned(
            field_attr,
            "field must have one of: required, optional, or default = value",
        ));
    };

    Ok(FieldConfig {
        env_var,
        description,
        example,
        mode,
    })
}

/// Extract the inner type from ConfigField<T> or just return the type as-is
fn extract_inner_type(ty: &Type) -> &Type {
    // For now, just return the type as-is since we're wrapping it ourselves
    ty
}

/// Extract the inner type from Option<T>, returns (is_option, inner_type)
fn extract_option_type(ty: &Type) -> (bool, &Type) {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
    {
        return (true, inner_ty);
    }
    (false, ty)
}
