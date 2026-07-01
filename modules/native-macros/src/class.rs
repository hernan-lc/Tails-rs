use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, Type};

fn snake_to_camel(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

fn rust_type_to_ts(ty: &Type) -> String {
    match ty {
        Type::Path(tp) => {
            if let Some(segment) = tp.path.segments.last() {
                let name = segment.ident.to_string();
                match name.as_str() {
                    "String" | "str" => "string".to_string(),
                    "f64" | "f32" | "i64" | "i32" | "u64" | "u32" => "number".to_string(),
                    "bool" => "boolean".to_string(),
                    "Self" => "void".to_string(),
                    "()" => "void".to_string(),
                    _ => "unknown".to_string(),
                }
            } else {
                "unknown".to_string()
            }
        }
        Type::Tuple(tuple) => {
            if tuple.elems.is_empty() {
                "void".to_string()
            } else {
                "unknown".to_string()
            }
        }
        _ => "unknown".to_string(),
    }
}

pub fn expand_class_struct(item_impl: syn::ItemImpl) -> TokenStream {
    let self_type = &item_impl.self_ty;
    let methods = &item_impl.items;

    let struct_name = match &**self_type {
        Type::Path(tp) => tp.path.segments.last().unwrap().ident.clone(),
        _ => panic!("#[tails_class] must be on an impl block for a named struct"),
    };

    let registry_name = format_ident!("__TAILS_CLASS_INSTANCES_{}", struct_name);
    let with_instances_name = format_ident!("__tails_class_with_instances_{}", struct_name);
    let next_id_name = format_ident!("__tails_class_next_id_{}", struct_name);
    let init_fn_name = format_ident!("__tails_class_init_{}", struct_name);

    let mut ffi_functions = Vec::new();
    let mut register_calls = Vec::new();
    let mut dts_entries = Vec::new();

    for item in methods {
        if let syn::ImplItem::Fn(method) = item {
            let method_name = &method.sig.ident;
            let method_name_str = method_name.to_string();
            let ffi_name = format_ident!("__tails_ffi_{}_{}", struct_name, method_name);

            let is_constructor = match &method.sig.output {
                syn::ReturnType::Type(_, ty) => {
                    matches!(ty.as_ref(), Type::Path(tp) if tp.path.segments.last().map(|s| s.ident == "Self").unwrap_or(false))
                }
                _ => false,
            };

            let mut param_names = Vec::new();
            let mut param_types = Vec::new();
            let mut param_indices = Vec::new();

            for (i, arg) in method.sig.inputs.iter().enumerate() {
                if let FnArg::Typed(pt) = arg {
                    if let syn::Pat::Ident(ident) = &*pt.pat {
                        param_names.push(ident.ident.clone());
                        param_types.push(pt.ty.clone());
                        param_indices.push(i);
                    }
                }
            }

            let body = &method.block;

            if is_constructor {
                ffi_functions.push(quote! {
                    #[no_mangle]
                    pub extern "C" fn #ffi_name(
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

                        #(
                            let #param_names = if let Some(arg) = args_slice.get(#param_indices) {
                                <#param_types as ::tails_abi::FromNativeValue>::from_native_value(*arg)
                                    .unwrap_or_default()
                            } else {
                                Default::default()
                            };
                        )*

                        let instance = { #body };

                        let id = #next_id_name();
                        #with_instances_name(|map| {
                            map.insert(id, instance);
                        });

                        ::tails_abi::NativeValue {
                            tag: 5,
                            data: id as u64,
                        }
                    }
                });
            } else {
                ffi_functions.push(quote! {
                    #[no_mangle]
                    pub extern "C" fn #ffi_name(
                        _interp: *mut ::std::ffi::c_void,
                        this: ::tails_abi::NativeValue,
                        args: *const ::tails_abi::NativeValue,
                        argc: i32,
                    ) -> ::tails_abi::NativeValue {
                        let id = this.data as u32;
                        let args_slice = if args.is_null() || argc <= 0 {
                            &[]
                        } else {
                            unsafe { ::std::slice::from_raw_parts(args, argc as usize) }
                        };

                        #(
                            let #param_names = if let Some(arg) = args_slice.get(#param_indices) {
                                <#param_types as ::tails_abi::FromNativeValue>::from_native_value(*arg)
                                    .unwrap_or_default()
                            } else {
                                Default::default()
                            };
                        )*

                        #with_instances_name(|map| {
                            if let Some(instance) = map.get_mut(&id) {
                                let result = instance.#method_name(#(#param_names),*);
                                ::tails_abi::ToNativeValue::to_native_value(&result)
                                    .unwrap_or_else(|_| ::tails_abi::NativeValue { tag: 0, data: 0 })
                            } else {
                                ::tails_abi::NativeValue { tag: 0, data: 0 }
                            }
                        })
                    }
                });
            }

            let struct_name_lower = struct_name.to_string().to_lowercase();
            let js_method_name = snake_to_camel(&method_name_str);
            let js_name = if is_constructor {
                struct_name.to_string()
            } else {
                format!("{}_{}", struct_name_lower, js_method_name)
            };

            register_calls.push(quote! {
                handle.module.register(#js_name, #ffi_name as ::tails_abi::NativeFn);
            });

            // Generate DTS metadata
            let param_dts: Vec<String> = param_names
                .iter()
                .zip(param_types.iter())
                .map(|(name, ty)| format!("{}: {}", name, rust_type_to_ts(ty)))
                .collect();
            let ret_ts = match &method.sig.output {
                syn::ReturnType::Default => "void".to_string(),
                syn::ReturnType::Type(_, ty) => {
                    if is_constructor {
                        struct_name.to_string()
                    } else {
                        rust_type_to_ts(ty)
                    }
                }
            };
            let dts_name =
                format!("__TAILS_DTS_{}_{}", struct_name, method_name_str).to_uppercase();
            let dts_sig = if is_constructor {
                format!(
                    "export function {}({}): {};",
                    js_name,
                    param_dts.join(", "),
                    ret_ts
                )
            } else {
                format!(
                    "export function {}({}): {};",
                    js_name,
                    param_dts.join(", "),
                    ret_ts
                )
            };
            let dts_ident = format_ident!("{}", dts_name);
            dts_entries.push(quote! {
                #[used]
                #[doc(hidden)]
                #[no_mangle]
                pub static #dts_ident: &str = #dts_sig;
            });
        }
    }

    quote! {
        #[allow(dead_code)]
        #item_impl

        #[allow(dead_code, non_snake_case, non_upper_case_globals)]
        static #registry_name: ::std::sync::Mutex<Option<::std::collections::HashMap<u32, #struct_name>>> =
            ::std::sync::Mutex::new(None);

        #[allow(non_snake_case, non_upper_case_globals)]
        fn #next_id_name() -> u32 {
            use ::std::sync::atomic::{AtomicU32, Ordering};
            static COUNTER: AtomicU32 = AtomicU32::new(1);
            COUNTER.fetch_add(1, Ordering::SeqCst)
        }

        #[allow(non_snake_case)]
        pub(crate) fn #with_instances_name<F, R>(f: F) -> R
        where
            F: FnOnce(&mut ::std::collections::HashMap<u32, #struct_name>) -> R,
        {
            let mut guard = #registry_name.lock().unwrap();
            if guard.is_none() {
                *guard = Some(::std::collections::HashMap::new());
            }
            f(guard.as_mut().unwrap())
        }

        #(#ffi_functions)*

        #(#dts_entries)*

        #[allow(non_snake_case)]
        fn #init_fn_name(handle: &mut ::tails_abi::ModuleHandle) {
            #(#register_calls)*
        }
    }
}
