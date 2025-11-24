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

    let mut value_field_defs = Vec::new(); // For Config struct (direct values)
    let mut meta_field_defs = Vec::new(); // For ConfigMeta struct (metadata)
    let mut load_impl_fields = Vec::new();
    let mut load_impl_unwraps = Vec::new();
    let mut docs_impl_fields = Vec::new();
    let mut meta_field_inits = Vec::new(); // For initializing ConfigMeta fields

    for field in fields {
        let field_name = field
            .ident
            .as_ref()
            .expect("BUG: field must have a name (already validated that struct has named fields)");
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

        // Generate direct value field for Config struct
        let inner_type = extract_inner_type(field_type);
        value_field_defs.push(quote! {
            #(#cfg_attrs)*
            #field_vis #field_name: #field_type
        });

        // For optional fields (Option<T>), metadata should use the inner type T
        // For other fields, use the full type
        let (is_option, opt_inner_type) = extract_option_type(field_type);
        let meta_type = if is_option {
            opt_inner_type
        } else {
            field_type
        };

        // Generate metadata field for ConfigMeta struct
        meta_field_defs.push(quote! {
            #(#cfg_attrs)*
            #field_vis #field_name: ::config_loadr::ConfigFieldMeta<#meta_type>
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
                // For optional fields, use the inner type (without Option wrapper)
                let opt_inner = if is_option {
                    actual_type
                } else {
                    return Err(syn::Error::new_spanned(
                        field,
                        "optional fields must have type Option<T>",
                    ));
                };

                // Example is optional - only used for documentation
                // Type is inferred from the field's Option<T> annotation
                quote! {
                    #(#cfg_attrs)*
                    let #field_name = builder.optional::<#opt_inner>(
                        #env_var,
                        #description,
                        None,
                    );
                }
            }
        };

        load_impl_fields.push(load_code.clone());

        // For all fields, unwrap the Option<T> returned by builder
        let unwrap_code = if matches!(config.mode, FieldMode::Optional) {
            // Optional fields return Option<T> from builder, assign directly
            quote! {
                #(#cfg_attrs)*
                #field_name
            }
        } else {
            // Required and default fields return Option<T>, unwrap them
            quote! {
                #(#cfg_attrs)*
                #field_name: #field_name.expect(concat!("BUG: field '", stringify!(#field_name), "' should have a value after finish()"))
            }
        };
        load_impl_unwraps.push(unwrap_code);

        // Generate metadata field initialization
        let meta_init = match &config.mode {
            FieldMode::Required => {
                let example = config.example.as_ref().unwrap();
                quote! {
                    #(#cfg_attrs)*
                    #field_name: ::config_loadr::ConfigFieldMeta::required(
                        #env_var,
                        #description,
                        #example,
                    )
                }
            }
            FieldMode::Default(default_expr) => {
                quote! {
                    #(#cfg_attrs)*
                    #field_name: ::config_loadr::ConfigFieldMeta::optional(
                        #env_var,
                        #description,
                        #default_expr,
                    )
                }
            }
            FieldMode::Optional => {
                // Use example if provided, otherwise use Default::default()
                let example_value = config
                    .example
                    .as_ref()
                    .map(|ex| quote! { #ex })
                    .unwrap_or_else(|| quote! { Default::default() });

                quote! {
                    #(#cfg_attrs)*
                    #field_name: ::config_loadr::ConfigFieldMeta::optional(
                        #env_var,
                        #description,
                        #example_value,
                    )
                }
            }
        };
        meta_field_inits.push(meta_init);

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
                if let Ok(ident) = attr.parse_args::<syn::Ident>() {
                    if ident == "missing_docs" {
                        return false;
                    }
                }
            }
            true
        })
        .collect();

    // Generate the Config struct definition (with direct values)
    let struct_def = quote! {
        #(#filtered_attrs)*
        #vis struct #struct_name {
            #(#value_field_defs),*
        }
    };

    // Generate the ConfigMeta struct name and definition
    let meta_struct_name = syn::Ident::new(&format!("{}Meta", struct_name), struct_name.span());
    let meta_struct_def = quote! {
        #[allow(missing_docs)]
        #vis struct #meta_struct_name {
            #(#meta_field_defs),*
        }
    };

    // Generate static metadata instance with unique name per config
    let meta_static_name = syn::Ident::new(
        &format!("__CONFIG_META_{}", struct_name.to_string().to_uppercase()),
        struct_name.span(),
    );
    let meta_static = quote! {
        static #meta_static_name: ::std::sync::OnceLock<#meta_struct_name> = ::std::sync::OnceLock::new();
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

            fn new() -> Result<Self, Vec<::config_loadr::ConfigError>> {
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

    // Generate inherent methods that delegate to the trait implementation
    // This allows users to call Config::load() without importing the Load trait
    let inherent_impl = quote! {
        impl #struct_name {
            /// Loads the configuration from environment variables.
            /// Panics if any required variables are missing or invalid.
            #vis fn load() -> Self {
                <Self as ::config_loadr::Load>::load()
            }

            /// Loads the configuration from environment variables.
            /// Returns an error if any required variables are missing or invalid.
            #vis fn new() -> Result<Self, Vec<::config_loadr::ConfigError>> {
                <Self as ::config_loadr::Load>::new()
            }

            /// Creates a builder for documentation purposes only.
            #[doc(hidden)]
            #vis fn builder_for_docs() -> ::config_loadr::ConfigBuilder {
                <Self as ::config_loadr::Load>::builder_for_docs()
            }

            /// Returns a reference to the configuration metadata.
            #vis fn metadata() -> &'static #meta_struct_name {
                #meta_static_name.get_or_init(|| {
                    #meta_struct_name {
                        #(#meta_field_inits),*
                    }
                })
            }
        }
    };

    Ok(quote! {
        #struct_def
        #meta_struct_def
        #meta_static
        #load_impl
        #inherent_impl
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
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return (true, inner_ty);
                    }
                }
            }
        }
    }
    (false, ty)
}
