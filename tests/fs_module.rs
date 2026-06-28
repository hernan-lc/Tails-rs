use std::path::Path;
use tails::TailsRuntime;

#[test]
fn test_fs_write_and_read() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import fs from "./fs.native";
        fs.writeFileSync("/tmp/test_fs_write.txt", "Hello");
        fs.readFileSync("/tmp/test_fs_write.txt");
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String("Hello".to_string()));
    std::fs::remove_file("/tmp/test_fs_write.txt").ok();
}

#[test]
fn test_fs_exists_sync() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import fs from "./fs.native";
        fs.existsSync("/tmp/nonexistent_file_12345.txt");
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(false));
}

#[test]
fn test_fs_mkdir_and_readdir() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import fs from "./fs.native";
        fs.mkdirSync("/tmp/test_fs_mkdir", { recursive: true });
        fs.writeFileSync("/tmp/test_fs_mkdir/a.txt", "a");
        fs.writeFileSync("/tmp/test_fs_mkdir/b.txt", "b");
        let files = fs.readdirSync("/tmp/test_fs_mkdir");
        fs.rmSync("/tmp/test_fs_mkdir", { recursive: true });
        files.length;
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(2.0));
}

#[test]
fn test_fs_stat_sync() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import fs from "./fs.native";
        fs.writeFileSync("/tmp/test_fs_stat.txt", "test content");
        let stat = fs.statSync("/tmp/test_fs_stat.txt");
        fs.unlinkSync("/tmp/test_fs_stat.txt");
        stat.size;
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Integer(12));
}

#[test]
fn test_fs_stat_is_file() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import fs from "./fs.native";
        fs.writeFileSync("/tmp/test_fs_stat2.txt", "test");
        let stat = fs.statSync("/tmp/test_fs_stat2.txt");
        fs.unlinkSync("/tmp/test_fs_stat2.txt");
        stat.isFile;
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_fs_append_file() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import fs from "./fs.native";
        fs.writeFileSync("/tmp/test_fs_append.txt", "Hello");
        fs.appendFileSync("/tmp/test_fs_append.txt", " World");
        let result = fs.readFileSync("/tmp/test_fs_append.txt");
        fs.unlinkSync("/tmp/test_fs_append.txt");
        result;
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String("Hello World".to_string()));
}

#[test]
fn test_fs_copy_file() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import fs from "./fs.native";
        fs.writeFileSync("/tmp/test_fs_copy_src.txt", "copy me");
        fs.copyFileSync("/tmp/test_fs_copy_src.txt", "/tmp/test_fs_copy_dst.txt");
        let result = fs.readFileSync("/tmp/test_fs_copy_dst.txt");
        fs.unlinkSync("/tmp/test_fs_copy_src.txt");
        fs.unlinkSync("/tmp/test_fs_copy_dst.txt");
        result;
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String("copy me".to_string()));
}

#[test]
fn test_fs_rename_file() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import fs from "./fs.native";
        fs.writeFileSync("/tmp/test_fs_rename_old.txt", "rename me");
        fs.renameSync("/tmp/test_fs_rename_old.txt", "/tmp/test_fs_rename_new.txt");
        let result = fs.readFileSync("/tmp/test_fs_rename_new.txt");
        fs.unlinkSync("/tmp/test_fs_rename_new.txt");
        result;
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String("rename me".to_string()));
}

#[test]
fn test_fs_unlink_file() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import fs from "./fs.native";
        fs.writeFileSync("/tmp/test_fs_unlink.txt", "delete me");
        fs.unlinkSync("/tmp/test_fs_unlink.txt");
        fs.existsSync("/tmp/test_fs_unlink.txt");
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(false));
}

#[test]
fn test_fs_rm_recursive() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import fs from "./fs.native";
        fs.mkdirSync("/tmp/test_fs_rm_dir");
        fs.writeFileSync("/tmp/test_fs_rm_dir/file.txt", "data");
        fs.rmSync("/tmp/test_fs_rm_dir", { recursive: true });
        fs.existsSync("/tmp/test_fs_rm_dir");
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(false));
}
