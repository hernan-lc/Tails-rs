mod array_fns;
mod assert_fns;
mod buffer_fns;
mod child_process_fns;
mod collection_fns;
pub mod console;
pub(crate) mod constants;
mod crypto_fns;
mod date_fns;
mod encoding_fns;
mod error_fns;
mod events_fns;
mod fetch_fns;

/// Unified macro for generating stub implementations when a feature is disabled.
///
/// Generates multiple pub(super) fn stubs that return a RuntimeError indicating
/// the module is not enabled. This replaces the previous pattern of separate
/// `fs_stub!`, `http_stub!`, `net_stub!` macros.
///
/// # Example
///
/// ```ignore
/// disabled_module_stub!("fs",
///     native_fs_read_file_sync,
///     native_fs_write_file_sync,
/// );
/// ```
///
/// This generates:
/// ```ignore
/// pub(super) fn native_fs_read_file_sync(...) -> Result<Value> {
///     Err(Error::RuntimeError("fs module is not enabled...".into()))
/// }
/// ```
#[macro_export]
macro_rules! disabled_module_stub {
    (
        $module:literal,
        $($name:ident),* $(,)?
    ) => {
        $(
            pub(super) fn $name(
                _interp: &mut Interpreter,
                _this: &Value,
                _args: &[Value],
            ) -> Result<Value> {
                Err(Error::RuntimeError(
                    format!("{} module is not enabled. Rebuild with --features {}",
                        $module, $module
                    ).into()
                ))
            }
        )*
    };
}

#[cfg(feature = "fs")]
mod fs_fns;
#[cfg(not(feature = "fs"))]
mod fs_fns {
    use crate::errors::{Error, Result};
    use crate::objects::Value;
    use crate::vm::interpreter::Interpreter;

    disabled_module_stub!(
        "fs",
        native_fs_read_file_sync,
        native_fs_write_file_sync,
        native_fs_exists_sync,
        native_fs_mkdir_sync,
        native_fs_readdir_sync,
        native_fs_stat_sync,
        native_fs_unlink_sync,
        native_fs_rm_sync,
        native_fs_copy_file_sync,
        native_fs_rename_sync,
        native_fs_append_file_sync,
        native_fs_readdir,
        native_fs_read_file,
        native_fs_write_file,
        native_fs_stat,
        native_fs_mkdir,
        native_fs_unlink,
        native_fs_copy_file,
        native_fs_rename,
    );
}
mod function_fns;
mod generator_fns;
#[cfg(feature = "http")]
mod http_fns;
#[cfg(not(feature = "http"))]
mod http_fns {
    use crate::errors::{Error, Result};
    use crate::objects::Value;
    use crate::vm::interpreter::Interpreter;

    disabled_module_stub!(
        "http",
        native_http_create_server,
        native_http_server_listen,
        native_http_server_close,
        native_http_req_on,
        native_http_res_write_head,
        native_http_res_write,
        native_http_res_end,
        native_http_res_set_header,
        native_http_res_get_header,
        native_http_res_remove_header,
    );
}
#[cfg(feature = "net")]
mod net_fns;
#[cfg(not(feature = "net"))]
mod net_fns {
    use crate::errors::{Error, Result};
    use crate::objects::Value;
    use crate::vm::interpreter::Interpreter;

    disabled_module_stub!(
        "net",
        native_net_create_connection,
        native_net_socket_write,
        native_net_socket_end,
        native_net_socket_on,
    );
}
mod global_fns;
mod helpers;
mod intl_fns;
mod iterator_fns;
mod json_fns;
mod math_fns;
mod number_fns;
mod object_fns;
#[cfg(feature = "os")]
mod os_fns;
#[cfg(not(feature = "os"))]
mod os_fns {
    use crate::errors::{Error, Result};
    use crate::objects::Value;
    use crate::vm::interpreter::Interpreter;

    disabled_module_stub!(
        "os",
        native_os_platform,
        native_os_arch,
        native_os_cpus,
        native_os_totalmem,
        native_os_freemem,
        native_os_uptime,
        native_os_hostname,
        native_os_type,
        native_os_release,
        native_os_homedir,
        native_os_tmpdir,
    );
}
#[cfg(feature = "path")]
mod path_fns;
#[cfg(not(feature = "path"))]
mod path_fns {
    use crate::errors::{Error, Result};
    use crate::objects::Value;
    use crate::vm::interpreter::Interpreter;

    disabled_module_stub!(
        "path",
        native_path_join,
        native_path_resolve,
        native_path_basename,
        native_path_dirname,
        native_path_extname,
        native_path_relative,
        native_path_is_absolute,
        native_path_normalize,
    );
}
#[cfg(feature = "process")]
pub mod process_fns;
#[cfg(not(feature = "process"))]
pub mod process_fns {
    use crate::errors::{Error, Result};
    use crate::objects::Value;
    use crate::vm::interpreter::Interpreter;

    disabled_module_stub!(
        "process",
        native_process_exit,
        native_process_cwd,
        native_process_chdir,
        native_process_stdout_write,
        native_process_hrtime,
        native_process_hrtime_bigint,
        native_process_next_tick,
        native_process_kill,
        native_process_uptime,
        native_process_memory_usage,
        native_process_on,
        native_tty_isatty,
    );

    #[inline]
    pub fn exit_requested() -> bool {
        false
    }
    #[inline]
    pub fn take_exit_code() -> i32 {
        0
    }
    #[inline]
    pub fn install_signal_handlers() {}
    #[inline]
    pub fn request_exit(_code: i32) {}
}
mod promise_fns;
mod proxy_fns;
mod querystring_fns;
mod reflect_fns;
mod regexp_fns;
mod require_fns;
mod stream_fns;
mod string_fns;
mod symbol_fns;
mod timers_fns;
mod typed_array_fns;
mod url_fns;
mod util_fns;
mod websocket_fns;
#[cfg(feature = "zlib")]
mod zlib_fns;
#[cfg(not(feature = "zlib"))]
mod zlib_fns {
    use crate::errors::{Error, Result};
    use crate::objects::Value;
    use crate::vm::interpreter::Interpreter;

    disabled_module_stub!(
        "zlib",
        native_zlib_gzip_sync,
        native_zlib_gunzip_sync,
        native_zlib_deflate_sync,
        native_zlib_inflate_sync,
        native_zlib_deflate_raw_sync,
        native_zlib_inflate_raw_sync,
        native_zlib_gzip,
        native_zlib_gunzip,
        native_zlib_deflate,
        native_zlib_inflate,
    );
}
#[cfg(feature = "tls")]
mod tls_fns;
#[cfg(not(feature = "tls"))]
mod tls_fns {
    use crate::errors::{Error, Result};
    use crate::objects::Value;
    use crate::vm::interpreter::Interpreter;

    disabled_module_stub!(
        "tls",
        native_tls_connect,
        native_tls_create_secure_context,
        native_tls_socket_write,
        native_tls_socket_end,
        native_tls_create_server,
    );
}
#[cfg(feature = "dns")]
mod dns_fns;
#[cfg(not(feature = "dns"))]
mod dns_fns {
    use crate::errors::{Error, Result};
    use crate::objects::Value;
    use crate::vm::interpreter::Interpreter;

    disabled_module_stub!(
        "dns",
        native_dns_resolve,
        native_dns_lookup,
        native_dns_resolve4,
        native_dns_resolve6,
        native_dns_resolve_mx,
    );
}

pub use constants::*;

use crate::errors::Result;
use crate::objects::Value;
use crate::vm::interpreter::Interpreter;

pub type NativeFn = fn(&mut Interpreter, &Value, &[Value]) -> Result<Value>;

pub static NATIVE_TABLE: &[NativeFn] = &[
    console::native_console_log,
    console::native_console_warn,
    console::native_console_error,
    console::native_console_info,
    object_fns::native_object_keys,
    object_fns::native_object_values,
    object_fns::native_object_entries,
    object_fns::native_object_assign,
    json_fns::native_json_parse,
    json_fns::native_json_stringify,
    global_fns::native_parse_int,
    global_fns::native_parse_float,
    global_fns::native_is_nan,
    global_fns::native_is_finite,
    global_fns::native_set_timeout,
    global_fns::native_set_interval,
    global_fns::native_clear_timeout,
    global_fns::native_clear_interval,
    math_fns::native_math_abs,
    math_fns::native_math_floor,
    math_fns::native_math_ceil,
    math_fns::native_math_round,
    math_fns::native_math_min,
    math_fns::native_math_max,
    math_fns::native_math_random,
    math_fns::native_math_pow,
    math_fns::native_math_sqrt,
    math_fns::native_math_log,
    math_fns::native_math_sin,
    math_fns::native_math_cos,
    math_fns::native_math_tan,
    array_fns::native_array_push,
    array_fns::native_array_pop,
    array_fns::native_array_shift,
    array_fns::native_array_unshift,
    array_fns::native_array_slice,
    array_fns::native_array_splice,
    array_fns::native_array_index_of,
    array_fns::native_array_includes,
    array_fns::native_array_find,
    array_fns::native_array_find_index,
    array_fns::native_array_map,
    array_fns::native_array_filter,
    array_fns::native_array_reduce,
    array_fns::native_array_for_each,
    array_fns::native_array_some,
    array_fns::native_array_every,
    array_fns::native_array_join,
    array_fns::native_array_reverse,
    array_fns::native_array_sort,
    array_fns::native_array_concat,
    array_fns::native_array_flat,
    string_fns::native_string_char_at,
    string_fns::native_string_char_code_at,
    string_fns::native_string_slice,
    string_fns::native_string_substring,
    string_fns::native_string_index_of,
    string_fns::native_string_includes,
    string_fns::native_string_replace,
    string_fns::native_string_split,
    string_fns::native_string_trim,
    string_fns::native_string_to_lower_case,
    string_fns::native_string_to_upper_case,
    string_fns::native_string_starts_with,
    string_fns::native_string_ends_with,
    string_fns::native_string_repeat,
    string_fns::native_string_pad_start,
    string_fns::native_string_pad_end,
    global_fns::native_number_parse_int,
    global_fns::native_number_parse_float,
    global_fns::native_number_is_nan,
    global_fns::native_number_is_finite,
    error_fns::native_error_constructor,
    error_fns::native_type_error_constructor,
    error_fns::native_reference_error_constructor,
    error_fns::native_syntax_error_constructor,
    error_fns::native_range_error_constructor,
    promise_fns::native_promise_constructor,
    promise_fns::native_promise_then,
    promise_fns::native_promise_catch,
    promise_fns::native_promise_finally,
    promise_fns::native_promise_resolve,
    promise_fns::native_promise_reject,
    promise_fns::native_promise_all,
    promise_fns::native_promise_race,
    proxy_fns::native_proxy_constructor,
    reflect_fns::native_reflect_get,
    reflect_fns::native_reflect_set,
    reflect_fns::native_reflect_has,
    reflect_fns::native_reflect_delete_property,
    reflect_fns::native_reflect_apply,
    reflect_fns::native_reflect_construct,
    reflect_fns::native_reflect_own_keys,
    reflect_fns::native_reflect_get_own_property_descriptor,
    reflect_fns::native_reflect_define_property,
    reflect_fns::native_reflect_get_prototype_of,
    reflect_fns::native_reflect_set_prototype_of,
    reflect_fns::native_reflect_is_extensible,
    reflect_fns::native_reflect_prevent_extensions,
    object_fns::native_object_define_property,
    object_fns::native_object_get_own_property_descriptor,
    object_fns::native_object_freeze,
    // TypedArray
    typed_array_fns::native_typed_array_constructor,
    typed_array_fns::native_typed_array_from,
    typed_array_fns::native_typed_array_of,
    typed_array_fns::native_typed_array_get,
    typed_array_fns::native_typed_array_set,
    typed_array_fns::native_typed_array_length,
    typed_array_fns::native_typed_array_byte_length,
    typed_array_fns::native_typed_array_byte_offset,
    typed_array_fns::native_typed_array_subarray,
    typed_array_fns::native_typed_array_slice,
    // Map
    collection_fns::native_map_constructor,
    collection_fns::native_map_get,
    collection_fns::native_map_set,
    collection_fns::native_map_has,
    collection_fns::native_map_delete,
    collection_fns::native_map_clear,
    collection_fns::native_map_size,
    collection_fns::native_map_for_each,
    collection_fns::native_map_keys,
    collection_fns::native_map_values,
    collection_fns::native_map_entries,
    // Set
    collection_fns::native_set_constructor,
    collection_fns::native_set_add,
    collection_fns::native_set_has,
    collection_fns::native_set_delete,
    collection_fns::native_set_clear,
    collection_fns::native_set_size,
    collection_fns::native_set_for_each,
    collection_fns::native_set_values,
    collection_fns::native_set_keys,
    collection_fns::native_set_entries,
    // WeakMap
    collection_fns::native_weakmap_constructor,
    collection_fns::native_weakmap_get,
    collection_fns::native_weakmap_set,
    collection_fns::native_weakmap_has,
    collection_fns::native_weakmap_delete,
    // WeakSet
    collection_fns::native_weakset_constructor,
    collection_fns::native_weakset_add,
    collection_fns::native_weakset_has,
    collection_fns::native_weakset_delete,
    // Generator
    generator_fns::native_generator_next,
    generator_fns::native_generator_return,
    generator_fns::native_generator_throw,
    // Object additional methods
    object_fns::native_object_is,
    object_fns::native_object_prevent_extensions,
    object_fns::native_object_is_extensible,
    object_fns::native_object_is_sealed,
    object_fns::native_object_is_frozen,
    object_fns::native_object_seal,
    // Symbol
    symbol_fns::native_symbol_constructor,
    symbol_fns::native_symbol_for,
    symbol_fns::native_symbol_key_for,
    // Function.prototype
    function_fns::native_function_call,
    function_fns::native_function_apply,
    function_fns::native_function_bind,
    // Array additional methods
    array_fns::native_array_copy_within,
    array_fns::native_array_fill,
    array_fns::native_array_find_last,
    array_fns::native_array_find_last_index,
    array_fns::native_array_flat_map,
    array_fns::native_array_last_index_of,
    array_fns::native_array_is_array,
    array_fns::native_array_from,
    array_fns::native_array_of,
    // Promise additional methods
    promise_fns::native_promise_all_settled,
    promise_fns::native_promise_any,
    promise_fns::native_promise_with_resolvers,
    // BigInt
    number_fns::native_bigint_constructor,
    // Date (170-213)
    date_fns::native_date_constructor,
    date_fns::native_date_now,
    date_fns::native_date_parse,
    date_fns::native_date_utc,
    date_fns::native_date_get_time,
    date_fns::native_date_get_full_year,
    date_fns::native_date_get_month,
    date_fns::native_date_get_date,
    date_fns::native_date_get_day,
    date_fns::native_date_get_hours,
    date_fns::native_date_get_minutes,
    date_fns::native_date_get_seconds,
    date_fns::native_date_get_milliseconds,
    date_fns::native_date_get_timezone_offset,
    date_fns::native_date_get_utc_full_year,
    date_fns::native_date_get_utc_month,
    date_fns::native_date_get_utc_date,
    date_fns::native_date_get_utc_day,
    date_fns::native_date_get_utc_hours,
    date_fns::native_date_get_utc_minutes,
    date_fns::native_date_get_utc_seconds,
    date_fns::native_date_get_utc_milliseconds,
    date_fns::native_date_set_time,
    date_fns::native_date_set_full_year,
    date_fns::native_date_set_month,
    date_fns::native_date_set_date,
    date_fns::native_date_set_hours,
    date_fns::native_date_set_minutes,
    date_fns::native_date_set_seconds,
    date_fns::native_date_set_milliseconds,
    date_fns::native_date_set_utc_full_year,
    date_fns::native_date_set_utc_month,
    date_fns::native_date_set_utc_date,
    date_fns::native_date_set_utc_hours,
    date_fns::native_date_set_utc_minutes,
    date_fns::native_date_set_utc_seconds,
    date_fns::native_date_set_utc_milliseconds,
    date_fns::native_date_to_string,
    date_fns::native_date_to_iso_string,
    date_fns::native_date_to_utc_string,
    date_fns::native_date_to_date_string,
    date_fns::native_date_to_time_string,
    date_fns::native_date_to_json,
    date_fns::native_date_value_of,
    // RegExp (214-227)
    regexp_fns::native_regexp_constructor,
    regexp_fns::native_regexp_test,
    regexp_fns::native_regexp_exec,
    regexp_fns::native_regexp_to_string,
    regexp_fns::native_regexp_source,
    regexp_fns::native_regexp_flags,
    regexp_fns::native_regexp_global,
    regexp_fns::native_regexp_ignore_case,
    regexp_fns::native_regexp_multiline,
    regexp_fns::native_regexp_dot_all,
    regexp_fns::native_regexp_unicode,
    regexp_fns::native_regexp_sticky,
    regexp_fns::native_regexp_last_index,
    // RegExp-aware String methods (228-230)
    regexp_fns::native_string_match,
    regexp_fns::native_string_replace,
    regexp_fns::native_string_search,
    // Iterator helpers (230-235)
    iterator_fns::native_iterator_map,
    iterator_fns::native_iterator_filter,
    iterator_fns::native_iterator_take,
    iterator_fns::native_iterator_drop,
    iterator_fns::native_iterator_for_each,
    iterator_fns::native_iterator_to_array,
    // Array[Symbol.iterator] (236)
    iterator_fns::native_array_iterator,
    // Encoding (237-238)
    encoding_fns::native_atob,
    encoding_fns::native_btoa,
    // Process (239-245)
    process_fns::native_process_exit,
    process_fns::native_process_cwd,
    process_fns::native_process_chdir,
    process_fns::native_process_stdout_write,
    process_fns::native_process_hrtime,
    process_fns::native_process_hrtime_bigint,
    process_fns::native_process_next_tick,
    // Buffer (246-260)
    buffer_fns::native_buffer_constructor,
    buffer_fns::native_buffer_alloc,
    buffer_fns::native_buffer_from,
    buffer_fns::native_buffer_concat,
    buffer_fns::native_buffer_is_buffer,
    buffer_fns::native_buffer_byte_length,
    buffer_fns::native_buffer_to_string,
    buffer_fns::native_buffer_write,
    buffer_fns::native_buffer_slice,
    buffer_fns::native_buffer_copy,
    buffer_fns::native_buffer_fill,
    buffer_fns::native_buffer_compare,
    buffer_fns::native_buffer_equals,
    buffer_fns::native_buffer_index_of,
    // Intl (261-265)
    intl_fns::native_datetime_format_constructor,
    intl_fns::native_number_format_constructor,
    intl_fns::native_datetime_format_format,
    intl_fns::native_datetime_format_format_to_parts,
    intl_fns::native_number_format_format,
    // Path (266-273)
    path_fns::native_path_join,
    path_fns::native_path_resolve,
    path_fns::native_path_basename,
    path_fns::native_path_dirname,
    path_fns::native_path_extname,
    path_fns::native_path_relative,
    path_fns::native_path_is_absolute,
    path_fns::native_path_normalize,
    // URL (284-295)
    url_fns::native_url_constructor,
    url_fns::native_url_to_string,
    url_fns::native_search_params_get,
    url_fns::native_search_params_get_all,
    url_fns::native_search_params_has,
    url_fns::native_search_params_set,
    url_fns::native_search_params_append,
    url_fns::native_search_params_delete,
    url_fns::native_search_params_to_string,
    url_fns::native_search_params_entries,
    url_fns::native_search_params_keys,
    url_fns::native_search_params_values,
    url_fns::native_search_params_for_each,
    // Fs (296-306)
    fs_fns::native_fs_read_file_sync,
    fs_fns::native_fs_write_file_sync,
    fs_fns::native_fs_exists_sync,
    fs_fns::native_fs_mkdir_sync,
    fs_fns::native_fs_readdir_sync,
    fs_fns::native_fs_stat_sync,
    fs_fns::native_fs_unlink_sync,
    fs_fns::native_fs_rm_sync,
    fs_fns::native_fs_copy_file_sync,
    fs_fns::native_fs_rename_sync,
    fs_fns::native_fs_append_file_sync,
    // Fetch (307-310)
    fetch_fns::native_fetch,
    fetch_fns::native_response_text,
    fetch_fns::native_response_json,
    fetch_fns::native_response_array_buffer,
    // TypedArray constructors (311-321)
    typed_array_fns::native_int8_array_constructor,
    typed_array_fns::native_uint8_array_constructor,
    typed_array_fns::native_uint8_clamped_array_constructor,
    typed_array_fns::native_int16_array_constructor,
    typed_array_fns::native_uint16_array_constructor,
    typed_array_fns::native_int32_array_constructor,
    typed_array_fns::native_uint32_array_constructor,
    typed_array_fns::native_float32_array_constructor,
    typed_array_fns::native_float64_array_constructor,
    typed_array_fns::native_bigint64_array_constructor,
    typed_array_fns::native_biguint64_array_constructor,
    // EventEmitter (322-326)
    events_fns::native_event_emitter_constructor,
    events_fns::native_event_emitter_on,
    events_fns::native_event_emitter_emit,
    events_fns::native_event_emitter_off,
    events_fns::native_event_emitter_listener_count,
    // Crypto hash methods (327-328)
    crypto_fns::native_crypto_hash_update,
    crypto_fns::native_crypto_hash_digest,
    // OS (329-339)
    os_fns::native_os_platform,
    os_fns::native_os_arch,
    os_fns::native_os_cpus,
    os_fns::native_os_totalmem,
    os_fns::native_os_freemem,
    os_fns::native_os_uptime,
    os_fns::native_os_hostname,
    os_fns::native_os_type,
    os_fns::native_os_release,
    os_fns::native_os_homedir,
    os_fns::native_os_tmpdir,
    // Crypto (340-342)
    crypto_fns::native_crypto_random_bytes,
    crypto_fns::native_crypto_random_uuid,
    crypto_fns::native_crypto_create_hash,
    // Async Fs (343-349)
    fs_fns::native_fs_readdir,
    fs_fns::native_fs_read_file,
    fs_fns::native_fs_write_file,
    fs_fns::native_fs_stat,
    fs_fns::native_fs_mkdir,
    fs_fns::native_fs_unlink,
    fs_fns::native_fs_copy_file,
    fs_fns::native_fs_rename,
    // Console additional methods (350-357)
    console::native_console_table,
    console::native_console_dir,
    console::native_console_group,
    console::native_console_group_end,
    console::native_console_group_collapsed,
    console::native_console_time,
    console::native_console_time_end,
    console::native_console_assert,
    console::native_console_clear,
    // WebSocket (359-363)
    websocket_fns::native_websocket_constructor,
    websocket_fns::native_websocket_send,
    websocket_fns::native_websocket_close,
    websocket_fns::native_websocket_add_event_listener,
    websocket_fns::native_websocket_remove_event_listener,
    // Assert (364-365)
    assert_fns::native_assert,
    assert_fns::native_assert_strict_equal,
    // URL additional (366-369)
    url_fns::native_url_search_params_constructor,
    url_fns::native_url_can_parse,
    url_fns::native_url_parse,
    url_fns::native_url_to_json,
    // Headers (370-379)
    fetch_fns::native_headers_constructor,
    fetch_fns::native_headers_append,
    fetch_fns::native_headers_get,
    fetch_fns::native_headers_set,
    fetch_fns::native_headers_has,
    fetch_fns::native_headers_delete,
    fetch_fns::native_headers_for_each,
    fetch_fns::native_headers_keys,
    fetch_fns::native_headers_values,
    fetch_fns::native_headers_entries,
    // Request (380)
    fetch_fns::native_request_constructor,
    // Response (381-385)
    fetch_fns::native_response_constructor,
    fetch_fns::native_response_json_static,
    fetch_fns::native_response_error,
    fetch_fns::native_response_redirect,
    fetch_fns::native_response_clone,
    // child_process (386-388)
    child_process_fns::native_child_process_exec_sync,
    child_process_fns::native_child_process_exec,
    child_process_fns::native_child_process_spawn,
    // Object.prototype methods
    object_fns::native_object_has_own_property,
    // CommonJS require()
    require_fns::native_require,
    // fileURLToPath (382)
    url_fns::native_url_file_url_to_path,
    // Number.prototype methods (383-390)
    number_fns::native_number_to_fixed,
    number_fns::native_number_to_string,
    number_fns::native_number_value_of,
    number_fns::native_number_to_exponential,
    number_fns::native_number_to_precision,
    number_fns::native_number_is_integer,
    number_fns::native_number_is_safe_integer,
    number_fns::native_number_parse_float,
    // Boolean.prototype methods (391-392)
    number_fns::native_boolean_to_string,
    number_fns::native_boolean_value_of,
    // String.prototype.matchAll (393)
    string_fns::native_string_match_all,
    // Generator[Symbol.iterator] (394)
    generator_fns::native_generator_symbol_iterator,
    // HTTP server (395-401)
    http_fns::native_http_create_server,
    http_fns::native_http_server_listen,
    http_fns::native_http_server_close,
    http_fns::native_http_req_on,
    http_fns::native_http_res_write_head,
    http_fns::native_http_res_write,
    http_fns::native_http_res_end,
    // Net (TCP client) (402-405)
    net_fns::native_net_create_connection,
    net_fns::native_net_socket_write,
    net_fns::native_net_socket_end,
    net_fns::native_net_socket_on,
    // Process API completeness (406-409) — appended to avoid
    // renumbering every existing entry.
    process_fns::native_process_kill,
    process_fns::native_process_uptime,
    process_fns::native_process_memory_usage,
    process_fns::native_process_on,
    // Buffer API completeness (410-411) — appended.
    buffer_fns::native_buffer_is_encoding,
    buffer_fns::native_buffer_transcode,
    // util module (412-415)
    util_fns::native_util_format,
    util_fns::native_util_inspect,
    util_fns::native_util_promisify,
    util_fns::native_util_callbackify,
    // events expansion (416-422)
    events_fns::native_event_emitter_prepend_listener,
    events_fns::native_event_emitter_once,
    events_fns::native_event_emitter_remove_all_listeners,
    events_fns::native_event_emitter_event_names,
    events_fns::native_event_emitter_get_max_listeners,
    events_fns::native_event_emitter_set_max_listeners,
    events_fns::native_event_emitter_prepend_once_listener,
    // timers module (423-424)
    timers_fns::native_set_immediate,
    timers_fns::native_clear_immediate,
    // querystring module (425-428)
    querystring_fns::native_querystring_parse,
    querystring_fns::native_querystring_stringify,
    querystring_fns::native_querystring_encode,
    querystring_fns::native_querystring_decode,
    // stream module (429-442)
    stream_fns::native_readable_stream_read,
    stream_fns::native_readable_stream_pipe,
    stream_fns::native_readable_stream_unpipe,
    stream_fns::native_readable_stream_push,
    stream_fns::native_readable_stream_destroy,
    stream_fns::native_writable_stream_write,
    stream_fns::native_writable_stream_end,
    stream_fns::native_writable_stream_destroy,
    stream_fns::native_writable_stream_cork,
    stream_fns::native_writable_stream_uncork,
    stream_fns::native_stream_constructor,
    stream_fns::native_passthrough_constructor,
    stream_fns::native_stream_pipeline,
    stream_fns::native_stream_finished,
    // zlib module (443-452)
    zlib_fns::native_zlib_gzip_sync,
    zlib_fns::native_zlib_gunzip_sync,
    zlib_fns::native_zlib_deflate_sync,
    zlib_fns::native_zlib_inflate_sync,
    zlib_fns::native_zlib_deflate_raw_sync,
    zlib_fns::native_zlib_inflate_raw_sync,
    zlib_fns::native_zlib_gzip,
    zlib_fns::native_zlib_gunzip,
    zlib_fns::native_zlib_deflate,
    zlib_fns::native_zlib_inflate,
    // tls module (453-457)
    tls_fns::native_tls_connect,
    tls_fns::native_tls_create_secure_context,
    tls_fns::native_tls_socket_write,
    tls_fns::native_tls_socket_end,
    tls_fns::native_tls_create_server,
    // dns module (458-462)
    dns_fns::native_dns_resolve,
    dns_fns::native_dns_lookup,
    dns_fns::native_dns_resolve4,
    dns_fns::native_dns_resolve6,
    dns_fns::native_dns_resolve_mx,
    // Object.create (463)
    object_fns::native_object_create,
    // Array constructor (464)
    array_fns::native_array_constructor,
    // String constructor (465)
    string_fns::native_string_constructor,
    // Number constructor (466)
    number_fns::native_number_constructor,
    // Boolean constructor (467)
    number_fns::native_boolean_constructor,
    // Array.prototype.at (468)
    array_fns::native_array_at,
    // Math.trunc (469)
    math_fns::native_math_trunc,
    // Math.sign (470)
    math_fns::native_math_sign,
    // Math.hypot (471)
    math_fns::native_math_hypot,
    // Function constructor (472)
    function_fns::native_function_constructor,
    // Object.getOwnPropertyDescriptors (473)
    object_fns::native_object_get_own_property_descriptors,
    // Object.defineProperties (474)
    object_fns::native_object_define_properties,
    // Object.fromEntries (475)
    object_fns::native_object_from_entries,
    // String.prototype.trimStart / trimEnd (476-477)
    string_fns::native_string_trim_start,
    string_fns::native_string_trim_end,
    // tty.isatty (478)
    process_fns::native_tty_isatty,
    // util.deprecate (479)
    util_fns::native_util_deprecate,
    // Error.captureStackTrace (480)
    error_fns::native_error_capture_stack_trace,
    // CallSite methods (481-489)
    error_fns::native_callsite_get_file_name,
    error_fns::native_callsite_get_line_number,
    error_fns::native_callsite_get_column_number,
    error_fns::native_callsite_is_eval,
    error_fns::native_callsite_get_function_name,
    error_fns::native_callsite_get_method_name,
    error_fns::native_callsite_get_type_name,
    error_fns::native_callsite_get_this,
    error_fns::native_callsite_to_string,
    // Object.prototype.toString / Function.prototype.toString
    object_fns::native_object_to_string,
    function_fns::native_function_to_string,
    object_fns::native_object_get_own_property_symbols,
    global_fns::native_decode_uri,
    global_fns::native_decode_uri_component,
    global_fns::native_encode_uri,
    global_fns::native_encode_uri_component,
    global_fns::native_eval,
    iterator_fns::native_string_iterator,
    global_fns::native_get_intrinsic,
    encoding_fns::native_text_encoder_constructor,
    encoding_fns::native_text_decoder_constructor,
    encoding_fns::native_text_encoder_encode,
    encoding_fns::native_text_decoder_decode,
    util_fns::native_util_inherits,
    object_fns::native_object_get_own_property_names,
    object_fns::native_object_has_own,
    global_fns::native_debug_noop,
    global_fns::native_debug_logger_noop,
    http_fns::native_http_res_set_header,
    http_fns::native_http_res_get_header,
    http_fns::native_http_res_remove_header,
    // Iterator.prototype.next (512)
    iterator_fns::native_iterator_next,
];

#[cfg(test)]
const _: () = assert!(!NATIVE_TABLE.is_empty(), "NATIVE_TABLE must not be empty");
