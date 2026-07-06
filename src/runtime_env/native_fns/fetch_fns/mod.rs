mod fetch;
mod headers;
mod request;
mod response;

pub(crate) use fetch::native_fetch;
pub(crate) use headers::{
    native_headers_append, native_headers_constructor, native_headers_delete,
    native_headers_entries, native_headers_for_each, native_headers_get, native_headers_has,
    native_headers_keys, native_headers_set, native_headers_values,
};
pub(crate) use request::native_request_constructor;
pub(crate) use response::{
    native_response_array_buffer, native_response_clone, native_response_constructor,
    native_response_error, native_response_json, native_response_json_static,
    native_response_redirect, native_response_text,
};
