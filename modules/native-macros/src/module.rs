use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, LitStr, Meta};

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

pub fn expand_module(item: ItemFn, options: ModuleOptions) -> TokenStream {
    let vis = &item.vis;
    let sig = &item.sig;
    let func_name = &sig.ident;
    let block = &item.block;
    let attrs = &item.attrs;

    let module_name = options.name.unwrap_or_else(|| {
        let name_str = func_name.to_string();
        if name_str == "init" || name_str == "new" {
            "module".to_string()
        } else {
            name_str
        }
    });

    let init_fn_name = format_ident!("tails_native_init");
    let meta_name = format_ident!(
        "__TAILS_MODULE_META_{}",
        module_name.replace('-', "_").to_uppercase()
    );

    let original_fn = quote! {
        #(#attrs)*
        #vis #sig {
            #block
        }
    };

    let ffi_fn = quote! {
        #[no_mangle]
        pub extern "C" fn #init_fn_name() -> *mut ::tails_abi::NativeModule {
            let module = ::tails_abi::NativeModule::new(#module_name);
            let mut handle = ::tails_abi::ModuleHandle::new(module);

            // The user's init function populates the module
            let user_module = #func_name();

            // Return the module as a raw pointer (ownership transferred)
            Box::into_raw(Box::new(handle))
        }
    };

    let meta_fn = quote! {
        #[used]
        #[doc(hidden)]
        #[no_mangle]
        pub static #meta_name: &str = #module_name;
    };

    quote! {
        #original_fn

        #ffi_fn

        #meta_fn
    }
}
