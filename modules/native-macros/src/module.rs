use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemMod, LitStr, Meta};

pub struct ModuleOptions {
    pub name: Option<String>,
}

pub fn parse_module_options(attrs: &[syn::Attribute]) -> ModuleOptions {
    let mut name = None;
    for attr in attrs {
        if attr.path().is_ident("tails") {
            if let Meta::List(list) = &attr.meta {
                if let Ok(nested) = &list.parse_nested_meta(|meta| {
                    if meta.path.is_ident("name") {
                        let value = meta.value()?;
                        let lit: LitStr = value.parse()?;
                        name = Some(lit.value());
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
    ModuleOptions { name }
}

pub fn expand_module(item: ItemMod, options: ModuleOptions) -> TokenStream {
    let vis = &item.vis;
    let mod_name = &item.ident;
    let content = item.content.as_ref().map(|(_, items)| items);
    let attrs = &item.attrs;

    let module_name = options
        .name
        .unwrap_or_else(|| mod_name.to_string().replace('_', "-").to_lowercase());

    let mut registrations = Vec::new();
    let mut function_items = Vec::new();

    if let Some(items) = content {
        for item in items {
            match item {
                syn::Item::Fn(func) => {
                    let func_name_str = func.sig.ident.to_string();

                    if func_name_str.starts_with("__tails_") {
                        function_items.push(quote! { #func });
                        continue;
                    }

                    // Detect whether this function has `#[tails_function]`. If
                    // so, rewrite the attribute to inject `module = "<name>"`
                    // (and preserve any explicit `js_name`) so the
                    // function-level macro generates module-scoped symbol names
                    // and avoids collisions when multiple native modules are
                    // linked into the same binary.
                    let mut new_func = func.clone();
                    let mut has_tails_function = false;
                    for attr in new_func.attrs.iter_mut() {
                        if attr.path().is_ident("tails_function") {
                            has_tails_function = true;
                            // Preserve an explicit `js_name` so aliases (e.g.
                            // camelCase `readFile` for snake_case `read_file`)
                            // are registered under the intended JS name.
                            let existing_js_name = extract_js_name(std::slice::from_ref(attr));
                            let new_attr_tokens = if let Some(js) = existing_js_name {
                                quote! {
                                    #[tails_function(module = #module_name, js_name = #js)]
                                }
                            } else {
                                quote! {
                                    #[tails_function(module = #module_name)]
                                }
                            };
                            *attr = syn::parse_quote!(#new_attr_tokens);
                        }
                    }

                    let actual_js_name =
                        extract_js_name(&new_func.attrs).unwrap_or_else(|| func_name_str.clone());

                    // Use the module-scoped FFI name to match the function
                    // macro's emission.
                    let safe_mod = module_name.replace('-', "_");
                    let ffi_name = format_ident!("__tails_{}_ffi_{}", safe_mod, new_func.sig.ident);
                    registrations.push(quote! {
                        handle.module.register(#actual_js_name, #ffi_name as ::tails_abi::NativeFn);
                    });

                    if has_tails_function {
                        // The function has been re-emitted with the rewritten
                        // attribute; pass it through so `#[tails_function]`
                        // expands with the module prefix.
                        function_items.push(quote! { #new_func });
                    } else {
                        function_items.push(quote! { #func });
                    }
                }
                syn::Item::Struct(s) => {
                    let struct_name = &s.ident;
                    let class_init = format_ident!("__tails_class_init_{}", struct_name);

                    registrations.push(quote! {
                        #class_init(&mut handle);
                    });

                    function_items.push(quote! { #s });
                }
                other => {
                    function_items.push(quote! { #other });
                }
            }
        }
    }

    let safe_module_name = module_name.replace('-', "_");
    let unique_init_name = format_ident!("tails_native_init_{}", safe_module_name);
    let meta_name = format_ident!("__TAILS_MODULE_META_{}", safe_module_name.to_uppercase());

    let meta_fn = quote! {
        #[used]
        #[doc(hidden)]
        #[no_mangle]
        pub static #meta_name: &str = #module_name;
    };

    let init_fn = quote! {
        #[no_mangle]
        pub extern "C" fn #unique_init_name() -> *mut ::tails_abi::ModuleHandle {
            let module = ::tails_abi::NativeModule::new(#module_name);
            let mut handle = ::tails_abi::ModuleHandle::new(module);

            #(#registrations)*

            Box::into_raw(Box::new(handle))
        }
    };

    quote! {
        #(#attrs)*
        #vis mod #mod_name {
            #(#function_items)*

            #init_fn
            #meta_fn
        }
    }
}

fn extract_js_name(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("tails") || attr.path().is_ident("tails_function") {
            if let Meta::List(list) = &attr.meta {
                let mut js_name = None;
                let _ = list.parse_nested_meta(|meta| {
                    if meta.path.is_ident("js_name") {
                        let value = meta.value()?;
                        let lit: LitStr = value.parse()?;
                        js_name = Some(lit.value());
                        Ok(())
                    } else {
                        Ok(())
                    }
                });
                if js_name.is_some() {
                    return js_name;
                }
            }
        }
    }
    None
}
