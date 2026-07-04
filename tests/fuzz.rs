//! Fuzzing harness skeleton for the compiler pipeline.
//!
//! This is a *bootstrap* fuzzing harness — it does not need any
//! external fuzzer dependency (no `cargo-fuzz`, no `proptest`) and
//! runs as a regular `cargo test`. The plan:
//!
//! 1. Generate a deterministic corpus of random byte sequences
//!    that cover the most common surface area of the language.
//! 2. For each input, run the full pipeline (`tokenize` →
//!    `compile`) and assert one of two outcomes:
//!    - `Ok` — the input is a valid program.
//!    - `Err` — the input is malformed; the compiler reports a clean
//!      error. (No panics, no infinite loops.)
//!
//! The key assertion is the *no-panic* invariant: malformed input
//! must always surface as a typed `Error`, never a Rust panic.
//!
//! To extend with coverage-guided mutation, swap the
//! `deterministic_corpus()` body for a `proptest` / `cargo-fuzz`
//! generator.

use tails::compiler::lexer::tokenize;
use tails::compiler::Compiler;

/// Deterministic corpus used by the harness. The label is only used
/// in panic messages to help locate the offending input.
const CORPUS: &[(&str, &[u8])] = &[
    // Empty / whitespace-only.
    ("empty", b""),
    ("ascii_ws", b"   \t\n\r\n  \t"),
    // ASCII identifiers and keywords.
    ("ident_simple", b"foo"),
    ("ident_camel", b"fooBarBaz"),
    ("ident_snake", b"foo_bar_baz"),
    ("keyword_let", b"let"),
    ("keyword_function", b"function f(){return 1}"),
    // Numbers and operators.
    ("int_lit", b"42"),
    ("float_lit", b"3.14"),
    ("bigint", b"42n"),
    // Strings.
    ("string_simple", b"\"hello\""),
    ("string_escapes", b"\"a\\\"b\\nc\""),
    ("string_template", b"`hello ${name}!`"),
    ("string_template_unterminated", b"`hello ${"),
    // Regex.
    ("regex", b"/foo/g"),
    // Punctuation.
    ("all_punct", b"!@#$%^&*()_+-=[]{}|;:',.<>?/`~"),
    // Real-looking programs.
    ("hello_world", b"console.log(\"hi\");"),
    ("arrow_fn", b"const f = (x) => x + 1;"),
    ("class_decl", b"class A { m(){ return 1; } }"),
    ("for_loop", b"for (let i=0;i<10;i++) { sum += i; }"),
    // Malformed: must surface as a clean error, never a panic.
    ("unterminated_string", b"\"abc"),
    ("unterminated_comment", b"/* abc"),
    ("unclosed_paren", b"(1 + 2"),
    ("unclosed_brace", b"{ let x = 1"),
    ("unexpected_token", b"@@@"),
    // Random bytes that almost certainly aren't valid UTF-8.
    ("random_bytes", &[0xFF, 0xFE, 0x00, 0x01, 0x80, 0x90]),
    // Long identifier.
    ("long_ident", b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
];

/// Decode `bytes` as UTF-8 (lossy) and run the full compiler
/// pipeline. The function only asserts that the pipeline does not
/// panic; the `Result` is otherwise ignored.
fn run_pipeline(bytes: &[u8]) {
    // `from_utf8_lossy` is the same behaviour the `tails` CLI uses
    // when a user passes a file that wasn't decoded as UTF-8.
    let source = String::from_utf8_lossy(bytes);
    let _ = tokenize(&source);
    // `type_checking = false` so the fuzzer doesn't depend on the
    // type-checker being able to handle arbitrary input — the
    // bytecode generator is what we want to stress here.
    let compiler = Compiler::new(false);
    let _ = compiler.compile(&source);
}

#[test]
fn fuzz_corpus_does_not_panic() {
    for (label, bytes) in CORPUS {
        let label = *label;
        let bytes: &[u8] = bytes;
        // `catch_unwind` is the standard way to assert
        // "this code must not panic" without aborting the whole
        // test binary.
        let result = std::panic::catch_unwind(|| run_pipeline(bytes));
        assert!(
            result.is_ok(),
            "compiler panicked on corpus entry {label:?}: {result:?}"
        );
    }
}

/// A tiny xorshift32 pseudo-random generator so the test is
/// deterministic across runs and CI without depending on any
/// external crate. The stream only needs to hit a wide range of
/// states; it is not used for security.
fn xorshift32(state: &mut u32) -> u8 {
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    (*state & 0xFF) as u8
}

#[test]
fn fuzz_lexer_never_panics_on_short_random_input() {
    let mut state: u32 = 0xDEAD_BEEF;
    for i in 0..512 {
        let len = (xorshift32(&mut state) as usize) % 256;
        let bytes: Vec<u8> = (0..len).map(|_| xorshift32(&mut state)).collect();
        let result = std::panic::catch_unwind(|| {
            let s = String::from_utf8_lossy(&bytes);
            let _ = tokenize(&s);
        });
        assert!(
            result.is_ok(),
            "lexer panicked on fuzz iteration {i} with input of length {len}"
        );
    }
}

#[test]
fn fuzz_full_pipeline_never_panics_on_short_random_input() {
    let mut state: u32 = 0xCAFE_F00D;
    let compiler = Compiler::new(false);
    for i in 0..256 {
        let len = (xorshift32(&mut state) as usize) % 200;
        let bytes: Vec<u8> = (0..len).map(|_| xorshift32(&mut state)).collect();
        let result = std::panic::catch_unwind(|| {
            let s = String::from_utf8_lossy(&bytes);
            let _ = compiler.compile(&s);
        });
        assert!(
            result.is_ok(),
            "compiler panicked on fuzz iteration {i} with input of length {len}"
        );
    }
}
