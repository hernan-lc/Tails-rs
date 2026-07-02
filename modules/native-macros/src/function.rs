use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::{ItemFn, LitStr, Meta};

use crate::types::{extract_params_from_sig, get_ret_type};

pub struct FnOptions {
    pub js_name: Option<String>,
    pub module: Option<String>,
}

/// Parses `module = "..."` and `js_name = "..."` from the raw attribute
/// argument tokens of `#[tails_function(...)]`. This is the path used by the
/// `#[tails_module]` macro when it injects the module name into each
/// function-level attribute.
pub fn parse_fn_options_from_attr(attr: &proc_macro::TokenStream) -> FnOptions {
    let mut js_name = None;
    let mut module = None;
    let tokens: TokenStream = attr.clone().into();
    // The attribute is just `(module = "...")`, so parse it as a punctuated
    // list of `MetaNameValue` items rather than a full `Meta` (which expects
    // a path prefix).
    let parser =
        syn::punctuated::Punctuated::<syn::MetaNameValue, syn::Token![,]>::parse_terminated;
    if let Ok(list) = parser.parse2(tokens) {
        for nv in list.iter() {
            if nv.path.is_ident("js_name") {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = &nv.value
                {
                    js_name = Some(s.value());
                }
            } else if nv.path.is_ident("module") {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = &nv.value
                {
                    module = Some(s.value());
                }
            }
        }
    }
    FnOptions { js_name, module }
}

pub fn parse_fn_options(attrs: &[syn::Attribute]) -> FnOptions {
    let mut js_name = None;
    let mut module = None;
    for attr in attrs {
        // The module-level macro injects options as a `#[tails_function(...)]`
        // attribute. Some authors may also use `#[tails(...)]` (alias) for
        // ergonomic reasons. Accept both.
        if attr.path().is_ident("tails_function") || attr.path().is_ident("tails") {
            if let Meta::List(list) = &attr.meta {
                if let Ok(_nested) = &list.parse_nested_meta(|meta| {
                    if meta.path.is_ident("js_name") {
                        let value = meta.value()?;
                        let lit: LitStr = value.parse()?;
                        js_name = Some(lit.value());
                        Ok(())
                    } else if meta.path.is_ident("module") {
                        let value = meta.value()?;
                        let lit: LitStr = value.parse()?;
                        module = Some(lit.value());
                        Ok(())
                    } else {
                        Err(meta.error("unknown tails attribute"))
                    }
                }) {
                    // ok
                }
            }
        }
    }
    FnOptions { js_name, module }
}

pub fn expand_function(item: ItemFn, options: FnOptions) -> TokenStream {
    let vis = &item.vis;
    let sig = &item.sig;
    let func_name = &sig.ident;
    let block = &item.block;
    let attrs = &item.attrs;

    let js_name = options.js_name.unwrap_or_else(|| func_name.to_string());

    // If a `module` is specified (set by `#[tails_module]`), the FFI and DTS
    // symbols are prefixed with `__tails_<module>_` so that multiple native
    // modules can be linked into the same binary without symbol collisions.
    let safe_mod = options
        .module
        .as_deref()
        .map(|m| m.replace('-', "_"))
        .unwrap_or_default();
    let wrapper_name = if safe_mod.is_empty() {
        format_ident!("__tails_ffi_{}", func_name)
    } else {
        format_ident!("__tails_{}_ffi_{}", safe_mod, func_name)
    };
    let meta_name_str = if safe_mod.is_empty() {
        format!("__TAILS_DTS_{}", func_name.to_string().to_uppercase())
    } else {
        format!(
            "__TAILS_{}_DTS_{}",
            safe_mod.to_uppercase(),
            func_name.to_string().to_uppercase()
        )
    };
    let meta_name = format_ident!("{}", meta_name_str);

    let params = extract_params_from_sig(sig);
    let ret_ts = get_ret_type(sig);

    let _param_decls: Vec<TokenStream> = params
        .iter()
        .map(|p| {
            let name = format_ident!("arg_{}", p.name);
            quote! { #name: ::tails_abi::NativeValue }
        })
        .collect();

    let param_conversions: Vec<TokenStream> = params
        .iter()
        .enumerate()
        .map(|(_i, p)| {
            let name = format_ident!("arg_{}", p.name);
            let rust_type = &p.rust_type;
            quote! {
                let #name = if let Some(arg) = args_slice.get(#_i) {
                    <#rust_type as ::tails_abi::FromNativeValue>::from_native_value(*arg)
                        .unwrap_or_default()
                } else {
                    Default::default()
                };
            }
        })
        .collect();

    let param_names: Vec<proc_macro2::Ident> = params
        .iter()
        .map(|p| format_ident!("arg_{}", p.name))
        .collect();

    let body_call = quote! {
        #func_name(#(#param_names),*)
    };

    let (ret_stmts, ret_expr) = match ret_ts.as_str() {
        "void" => (
            vec![quote! { #body_call; }],
            quote! { ::tails_abi::NativeValue { tag: 0, data: 0 } },
        ),
        _ => (
            vec![quote! {
                let result = #body_call;
            }],
            quote! {
                ::tails_abi::ToNativeValue::to_native_value(&result)
                    .map_err(|e| format!("{}: {}", stringify!(#func_name), e))?
            },
        ),
    };

    let param_dts: Vec<String> = params
        .iter()
        .map(|p| format!("{}: {}", p.name, p.ts_type))
        .collect();
    let dts_sig = format!(
        "export function {}({}): {};",
        js_name,
        param_dts.join(", "),
        ret_ts
    );
    let dts_lit = LitStr::new(&dts_sig, proc_macro2::Span::call_site());

    let original_fn = quote! {
        #(#attrs)*
        #vis #sig #block
    };

    let ffi_fn = quote! {
        #[allow(clippy::needless_question_mark)]
        #[no_mangle]
        pub extern "C" fn #wrapper_name(
            _interp: *mut ::std::ffi::c_void,
            _this: ::tails_abi::NativeValue,
            args: *const ::tails_abi::NativeValue,
            argc: i32,
        ) -> ::tails_abi::NativeValue {
            let args_slice = if args.is_null() || argc <= 0 {
                &[]
            } else {
                unsafe { ::std::slice::from_raw_parts(args, argc as usize) }
            };

            #(#param_conversions)*

            match (|| -> Result<::tails_abi::NativeValue, String> {
                #(#ret_stmts)*
                Ok(#ret_expr)
            })() {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("tails native error: {}", e);
                    ::tails_abi::NativeValue { tag: 0, data: 0 }
                }
            }
        }
    };

    let meta_fn = quote! {
        #[used]
        #[doc(hidden)]
        #[no_mangle]
        pub static #meta_name: &str = #dts_lit;
    };

    let _registration_entry = quote! {
        (#js_name, #wrapper_name as ::tails_abi::NativeFn)
    };

    quote! {
        #original_fn

        #ffi_fn

        #meta_fn
    }
}
