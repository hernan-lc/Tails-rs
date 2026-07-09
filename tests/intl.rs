use std::path::Path;
use tails::TailsRuntime;

#[test]
fn test_intl_datetime_format_default() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import Intl from "./intl.native";
        let dtf = new Intl.DateTimeFormat("en-US");
        let result = dtf.format();
        typeof result;
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("string"));
}

#[test]
fn test_intl_datetime_format_with_options() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import Intl from "./intl.native";
        let dtf = new Intl.DateTimeFormat("en-US", {
            year: "numeric",
            month: "long",
            day: "numeric"
        });
        let result = dtf.format();
        result.length > 0;
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_intl_datetime_format_to_parts() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import Intl from "./intl.native";
        let dtf = new Intl.DateTimeFormat("en-US", {
            year: "numeric",
            month: "long",
            day: "numeric"
        });
        let parts = dtf.formatToParts();
        parts.length > 0;
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_intl_number_format_default() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import Intl from "./intl.native";
        let nf = new Intl.NumberFormat("en-US");
        nf.format(1234567.89);
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("1,234,567.89"));
}

#[test]
fn test_intl_number_format_currency() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import Intl from "./intl.native";
        let nf = new Intl.NumberFormat("en-US", {
            style: "currency",
            currency: "USD"
        });
        nf.format(1234.56);
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("$1,234.56"));
}

#[test]
fn test_intl_number_format_percent() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import Intl from "./intl.native";
        let nf = new Intl.NumberFormat("en-US", {
            style: "percent"
        });
        nf.format(0.856);
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("85.6%"));
}

#[test]
fn test_intl_number_format_integer() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import Intl from "./intl.native";
        let nf = new Intl.NumberFormat("en-US");
        nf.format(1234567);
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("1,234,567"));
}

#[test]
fn test_intl_number_format_negative() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import Intl from "./intl.native";
        let nf = new Intl.NumberFormat("en-US");
        nf.format(-1234567.89);
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        tails::Value::string("-1,234,567.89")
    );
}
