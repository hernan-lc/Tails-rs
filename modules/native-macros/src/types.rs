use syn::{FnArg, Pat, Type, TypePath};

pub fn rust_type_to_ts(ty: &Type) -> String {
    match ty {
        Type::Path(tp) => path_type_to_ts(tp),
        Type::Reference(r) => rust_type_to_ts(&r.elem),
        Type::Tuple(tuple) => {
            if tuple.elems.is_empty() {
                "void".to_string()
            } else {
                let elems: Vec<String> = tuple.elems.iter().map(|e| rust_type_to_ts(e)).collect();
                format!("[{}]", elems.join(", "))
            }
        }
        Type::Slice(_) => "Array<unknown>".to_string(),
        Type::Array(_) => "Array<unknown>".to_string(),
        _ => "unknown".to_string(),
    }
}

fn path_type_to_ts(tp: &TypePath) -> String {
    if let Some(segment) = tp.path.segments.last() {
        let name = segment.ident.to_string();
        match name.as_str() {
            "String" | "str" => "string".to_string(),
            "f64" | "f32" | "i64" | "i32" | "i16" | "i8" | "u64" | "u32" | "u16" | "u8"
            | "isize" | "usize" => "number".to_string(),
            "bool" => "boolean".to_string(),
            "Vec" | "VecDeque" | "LinkedList" => {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                        return format!("Array<{}>", rust_type_to_ts(inner));
                    }
                }
                "Array<unknown>".to_string()
            }
            "Option" => {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                        return format!("{} | null", rust_type_to_ts(inner));
                    }
                }
                "unknown | null".to_string()
            }
            "HashMap" | "BTreeMap" => {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if args.args.len() >= 2 {
                        let key = if let Some(syn::GenericArgument::Type(k)) = args.args.get(0) {
                            rust_type_to_ts(k)
                        } else {
                            "string".to_string()
                        };
                        let val = if let Some(syn::GenericArgument::Type(v)) = args.args.get(1) {
                            rust_type_to_ts(v)
                        } else {
                            "unknown".to_string()
                        };
                        return format!("Record<{}, {}>", key, val);
                    }
                }
                "Record<string, unknown>".to_string()
            }
            "Result" => {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                        return rust_type_to_ts(inner);
                    }
                }
                "unknown".to_string()
            }
            "Box" => {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                        return rust_type_to_ts(inner);
                    }
                }
                "unknown".to_string()
            }
            "()" => "void".to_string(),
            _ => {
                if segment.arguments.is_empty() {
                    name
                } else {
                    "unknown".to_string()
                }
            }
        }
    } else {
        "unknown".to_string()
    }
}

pub struct ParamInfo {
    pub name: String,
    pub ts_type: String,
}

pub fn extract_params_from_sig(sig: &syn::Signature) -> Vec<ParamInfo> {
    let mut params = Vec::new();
    for arg in &sig.inputs {
        match arg {
            FnArg::Receiver(_) => {}
            FnArg::Typed(pat_type) => {
                let name = pat_to_name(&pat_type.pat);
                let ts_type = rust_type_to_ts(&pat_type.ty);
                params.push(ParamInfo { name, ts_type });
            }
        }
    }
    params
}

fn pat_to_name(pat: &Pat) -> String {
    match pat {
        Pat::Ident(ident) => ident.ident.to_string(),
        Pat::Wild(_) => "_".to_string(),
        _ => "unknown".to_string(),
    }
}

pub fn get_ret_type(sig: &syn::Signature) -> String {
    match &sig.output {
        syn::ReturnType::Default => "void".to_string(),
        syn::ReturnType::Type(_, ty) => rust_type_to_ts(&ty),
    }
}
