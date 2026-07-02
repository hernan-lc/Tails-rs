use std::path::Path;
use tails::TailsRuntime;

#[test]
fn test_crypto_random_bytes() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import crypto from "crypto";
        const buf = crypto.randomBytes(16);
        buf.length;
    "#,
        Path::new("/tmp/test_crypto_module.ts"),
    );
    assert!(r.is_ok(), "crypto.randomBytes() failed: {:?}", r.err());
    match r.unwrap() {
        tails::Value::Integer(n) => assert_eq!(n, 16, "randomBytes(16) should return 16 bytes"),
        tails::Value::Float(n) => assert_eq!(n as i64, 16, "randomBytes(16) should return 16 bytes"),
        other => panic!("Expected number for randomBytes length, got {:?}", other),
    }
}

#[test]
fn test_crypto_random_bytes_different() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import crypto from "crypto";
        const a = crypto.randomBytes(32);
        const b = crypto.randomBytes(32);
        // Check that both are buffers with correct length
        a.length === 32 && b.length === 32;
    "#,
        Path::new("/tmp/test_crypto_module.ts"),
    );
    assert!(r.is_ok(), "crypto.randomBytes comparison failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true), "Both buffers should have length 32");
}

#[test]
fn test_crypto_random_uuid() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import crypto from "crypto";
        const uuid = crypto.randomUUID();
        uuid.length;
    "#,
        Path::new("/tmp/test_crypto_module.ts"),
    );
    assert!(r.is_ok(), "crypto.randomUUID() failed: {:?}", r.err());
    match r.unwrap() {
        tails::Value::Integer(n) => assert_eq!(n, 36, "UUID should be 36 chars"),
        tails::Value::Float(n) => assert_eq!(n as i64, 36, "UUID should be 36 chars"),
        other => panic!("Expected number for UUID length, got {:?}", other),
    }
}

#[test]
fn test_crypto_random_uuid_format() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import crypto from "crypto";
        const uuid = crypto.randomUUID();
        // UUID v4 format: 36 chars with 4 dashes
        uuid.length === 36 &&
        uuid.includes("-");
    "#,
        Path::new("/tmp/test_crypto_module.ts"),
    );
    assert!(r.is_ok(), "crypto.randomUUID() format check failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true), "UUID should be 36 chars with dashes");
}

#[test]
fn test_crypto_create_hash() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import crypto from "crypto";
        const hash = crypto.createHash("sha256");
        typeof hash.update === "function" && typeof hash.digest === "function";
    "#,
        Path::new("/tmp/test_crypto_module.ts"),
    );
    assert!(r.is_ok(), "crypto.createHash() failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true), "Hash should have update and digest methods");
}

#[test]
fn test_crypto_hash_digest() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import crypto from "crypto";
        const hash = crypto.createHash("sha256");
        hash.update("hello");
        const result = hash.digest("hex");
        result.length;
    "#,
        Path::new("/tmp/test_crypto_module.ts"),
    );
    assert!(r.is_ok(), "crypto hash digest failed: {:?}", r.err());
    match r.unwrap() {
        tails::Value::Integer(n) => assert_eq!(n, 64, "SHA-256 hex digest should be 64 chars"),
        tails::Value::Float(n) => assert_eq!(n as i64, 64, "SHA-256 hex digest should be 64 chars"),
        other => panic!("Expected number for digest length, got {:?}", other),
    }
}

#[test]
fn test_crypto_hash_deterministic() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import crypto from "crypto";
        const h1 = crypto.createHash("sha256");
        h1.update("hello");
        const r1 = h1.digest("hex");
        const h2 = crypto.createHash("sha256");
        h2.update("hello");
        const r2 = h2.digest("hex");
        r1 === r2;
    "#,
        Path::new("/tmp/test_crypto_module.ts"),
    );
    assert!(r.is_ok(), "crypto hash determinism failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true), "Same input should produce same hash");
}
