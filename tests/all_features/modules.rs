use tails::{TailsRuntime, Value};

#[test]
fn test_buffer() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
    import Buffer from "./buffer.native";
    let buf = Buffer.from("Hello");
    let buf2 = Buffer.alloc(5, 65);
    let buf3 = Buffer.concat([buf, Buffer.from(" World")]);
    buf.toString() + "|" + buf.length + "|" + buf2.toString() + "|" + buf3.toString() + "|" + Buffer.isBuffer(buf);
    "#,
        std::path::Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        Value::String("Hello|5|AAAAA|Hello World|true".to_string())
    );
}

#[test]
#[cfg(feature = "process")]
fn test_process_globals() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
    import process from "process";
    typeof process.platform + "," + typeof process.arch + "," +
    typeof process.pid + "," + typeof process.cwd + "," + typeof process.env;
    "#,
        std::path::Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        Value::String("string,string,number,function,object".to_string())
    );
}

#[test]
#[cfg(feature = "path")]
fn test_path_module() {
    let mut rt = TailsRuntime::default();
    let sep = std::path::MAIN_SEPARATOR;
    let r = rt.eval(&format!(
        r#"
    import path from "path";
    path.join("/foo", "bar", "baz") + "," +
    path.basename("/foo/bar.txt", "") + "," +
    path.dirname("/foo/bar.txt") + "," +
    path.extname("/foo/bar.txt") + "," +
    path.isAbsolute("{abs_path}") + "," +
    path.normalize("/foo/../bar") + "," +
    (path.sep === "/" || path.sep === "\\");
    "#,
        abs_path = if sep == '\\' { "C:\\foo" } else { "/foo" },
    ));
    assert!(r.is_ok(), "path test failed: {:?}", r.err());
    assert_eq!(
        r.unwrap(),
        Value::String(format!(
            "/foo{sep}bar{sep}baz,bar.txt,/foo,.txt,true,/bar,true",
            sep = sep
        ))
    );
}

#[test]
#[cfg(feature = "fs")]
fn test_fs_module() {
    let tmp = std::env::temp_dir().to_string_lossy().to_string();
    let path = format!("{}/tails_test.txt", tmp);
    let mut rt = TailsRuntime::default();
    let r = rt.eval(&format!(
        r#"
    import fs from "fs";
    fs.writeFileSync("{path}", "Hello from Tails!");
    let read = fs.readFileSync("{path}");
    let exists1 = fs.existsSync("{path}");
    fs.unlinkSync("{path}");
    let exists2 = fs.existsSync("{path}");
    read + "," + exists1 + "," + exists2;
    "#,
        path = path,
    ));
    assert!(r.is_ok(), "fs test failed: {:?}", r.err());
    assert_eq!(
        r.unwrap(),
        Value::String("Hello from Tails!,true,false".to_string())
    );
}

#[test]
fn test_intl() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
    import Intl from "./intl.native";
    let dtf = new Intl.DateTimeFormat("en-US", { year: "numeric", month: "long", day: "numeric" });
    let nf = new Intl.NumberFormat("en-US");
    (dtf.format().length > 0) + "," + (nf.format(1234567.89).length > 0);
    "#,
        std::path::Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("true,true".to_string()));
}

#[test]
fn test_import_named() {
    let mut runtime = TailsRuntime::default();
    let source = r#"
        import { add, multiply } from "./tests/fixtures/modules/math.ts";
        add(10, 20)
    "#;
    let result = runtime.eval(source).unwrap();
    assert_eq!(result, tails::Value::Float(30.0));
}

#[test]
fn test_import_default() {
    let mut runtime = TailsRuntime::default();
    let source = r#"
        import greet from "./tests/fixtures/modules/greeter.ts";
        greet("World")
    "#;
    let result = runtime.eval(source).unwrap();
    assert_eq!(result, tails::Value::String("Hello, World!".to_string()));
}
