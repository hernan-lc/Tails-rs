mod dts;
mod function;
mod module;
mod types;

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

use function::expand_function;
use module::expand_module;

#[proc_macro_attribute]
pub fn tails_function(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(item as ItemFn);
    let options = function::parse_fn_options(&item_fn.attrs);
    expand_function(item_fn, options).into()
}

#[proc_macro_attribute]
pub fn tails_module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(item as ItemFn);
    let options = module::parse_module_options(&item_fn.attrs);
    expand_module(item_fn, options).into()
}

#[proc_macro_attribute]
pub fn function(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(item as ItemFn);
    let mut options = function::parse_fn_options(&item_fn.attrs);

    // Parse #[tails(js_name = "...")] from the attribute arguments
    if !attr.is_empty() {
        let attr_tokens: proc_macro2::TokenStream = attr.into();
        let attr_str = attr_tokens.to_string();
        if attr_str.contains("js_name") {
            if let Some(start) = attr_str.find("js_name") {
                let rest = &attr_str[start..];
                if let Some(eq_pos) = rest.find('=') {
                    let after_eq = rest[eq_pos + 1..].trim();
                    if after_eq.starts_with('"') && after_eq.ends_with('"') {
                        options.js_name = Some(after_eq[1..after_eq.len() - 1].to_string());
                    }
                }
            }
        }
    }

    expand_function(item_fn, options).into()
}

#[proc_macro_attribute]
pub fn module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(item as ItemFn);
    let mut options = module::parse_module_options(&item_fn.attrs);

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

    expand_module(item_fn, options).into()
}
