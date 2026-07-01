use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, LitStr, Meta};

use crate::types::{extract_params_from_sig, get_ret_type};

pub struct FnOptions {
    pub js_name: Option<String>,
}

pub fn parse_fn_options(attrs: &[syn::Attribute]) -> FnOptions {
    let mut js_name = None;
    for attr in attrs {
        if attr.path().is_ident("tails") {
            if let Meta::List(list) = &attr.meta {
                if let Ok(nested) = &list.parse_nested_meta(|meta| {
                    if meta.path.is_ident("js_name") {
                        let value = meta.value()?;
                        let lit: LitStr = value.parse()?;
                        js_name = Some(lit.value());
                        Ok(())
                    } else {
                        Err(meta.error("unknown tails attribute"))
                    }
                }) {
                    let _ = nested;
                }
            }
        }
    }
    FnOptions { js_name }
}

pub fn expand_function(item: ItemFn, options: FnOptions) -> TokenStream {
    let vis = &item.vis;
    let sig = &item.sig;
    let func_name = &sig.ident;
    let block = &item.block;
    let attrs = &item.attrs;

    let js_name = options.js_name.unwrap_or_else(|| func_name.to_string());

    let wrapper_name = format_ident!("__tails_ffi_{}", func_name);
    let meta_name_str = format!("__TAILS_DTS_{}", func_name.to_string().to_uppercase());
    let meta_name = format_ident!("{}", meta_name_str);

    let params = extract_params_from_sig(sig);
    let ret_ts = get_ret_type(sig);

    let _param_decls: Vec<TokenStream> = params
        .iter()
        .enumerate()
        .map(|(_i, p)| {
            let name = format_ident!("arg_{}", p.name);
            quote! { #name: ::tails_abi::NativeValue }
        })
        .collect();

    let param_conversions: Vec<TokenStream> = params
        .iter()
        .enumerate()
        .map(|(_i, p)| {
            let name = format_ident!("arg_{}", p.name);
            let ts_type = &p.ts_type;
            quote! {
                let #name = <#ts_type as ::tails_abi::FromNativeValue>::from_native_value(#name)
                    .map_err(|e| format!("{}: {}", stringify!(#func_name), e))?;
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

    let ret_conversion = match ret_ts.as_str() {
        "void" => quote! {
            #body_call;
            ::tails_abi::NativeValue { tag: 0, data: 0 }
        },
        _ => quote! {
            let result = #body_call;
            ::tails_abi::ToNativeValue::to_native_value(&result)
                .map_err(|e| format!("{}: {}", stringify!(#func_name), e))?
        },
    };

    let param_dts: Vec<String> = params
        .iter()
        .map(|p| format!("{}: {}", p.name, p.ts_type))
        .collect();
    let dts_sig = format!(
        "export function {}({}) => {};",
        js_name,
        param_dts.join(", "),
        ret_ts
    );
    let dts_lit = LitStr::new(&dts_sig, proc_macro2::Span::call_site());

    let original_fn = quote! {
        #(#attrs)*
        #vis #sig {
            #block
        }
    };

    let ffi_fn = quote! {
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
                Ok(#ret_conversion)
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
