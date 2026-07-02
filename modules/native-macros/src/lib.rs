mod class;
mod function;
mod module;
mod types;

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn, ItemImpl, ItemMod};

use class::expand_class_struct;
use function::expand_function;
use module::expand_module;

#[proc_macro_attribute]
pub fn tails_function(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(item as ItemFn);

    // Parse `module = "..."` and `js_name = "..."` from the attribute args.
    // These are typically injected by `#[tails_module]` but can also be
    // supplied directly by callers (e.g. when not using the module macro).
    let mut options = function::parse_fn_options_from_attr(&attr);

    // Also accept the legacy `#[tails(...)]` form attached to the function.
    let inner = function::parse_fn_options(&item_fn.attrs);
    if options.js_name.is_none() {
        options.js_name = inner.js_name;
    }
    if options.module.is_none() {
        options.module = inner.module;
    }

    expand_function(item_fn, options).into()
}

#[proc_macro_attribute]
pub fn tails_module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_mod = parse_macro_input!(item as ItemMod);
    let mut options = module::parse_module_options(&item_mod.attrs);

    if !attr.is_empty() {
        let attr_tokens: proc_macro2::TokenStream = attr.into();
        let attr_str = attr_tokens.to_string();
        if attr_str.contains("name") {
            if let Some(start) = attr_str.find("name") {
                let rest = &attr_str[start..];
                if let Some(eq_pos) = rest.find('=') {
                    let after_eq = rest[eq_pos + 1..].trim();
                    if after_eq.starts_with('"') && after_eq.ends_with('"') {
                        options.name = Some(after_eq[1..after_eq.len() - 1].to_string());
                    }
                }
            }
        }
    }

    expand_module(item_mod, options).into()
}

#[proc_macro_attribute]
pub fn tails_class(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_impl = parse_macro_input!(item as ItemImpl);
    expand_class_struct(item_impl).into()
}
