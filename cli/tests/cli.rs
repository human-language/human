use std::process::Command;

fn hmn() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hmn"))
}

fn fixtures() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

// ── help ──

#[test]
fn help_flag() {
    let out = hmn().arg("--help").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("validate"), "stderr: {stderr}");
    assert!(stderr.contains("compile"), "stderr: {stderr}");
    assert!(stderr.contains("fmt"), "stderr: {stderr}");
}

#[test]
fn help_short() {
    let out = hmn().arg("-h").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("validate"));
}

#[test]
fn no_args() {
    let out = hmn().output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("usage:"));
}

#[test]
fn unknown_command() {
    let out = hmn().arg("bogus").output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("unknown command: bogus"));
}

// ── validate ──

#[test]
fn validate_valid() {
    let out = hmn()
        .arg("validate")
        .arg(fixtures().join("valid.hmn"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stdout.is_empty());
    assert!(out.stderr.is_empty());
}

#[test]
fn validate_broken() {
    let out = hmn()
        .arg("validate")
        .arg(fixtures().join("broken.hmn"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("error:"), "stderr: {stderr}");
    assert!(stderr.contains(" --> "), "should have location arrow: {stderr}");
    assert!(stderr.contains(" | "), "should have gutter: {stderr}");
}

#[test]
fn validate_missing() {
    let out = hmn()
        .arg("validate")
        .arg("nonexistent.hmn")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("error:"), "stderr: {stderr}");
    assert!(stderr.contains(" --> "), "should have location arrow: {stderr}");
}

#[test]
fn validate_multiple() {
    let out = hmn()
        .arg("validate")
        .arg(fixtures().join("valid.hmn"))
        .arg(fixtures().join("broken.hmn"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("error:"), "stderr: {stderr}");
    assert!(!stderr.contains("valid.hmn"), "valid file should not appear in errors: {stderr}");
}

#[test]
fn validate_with_imports() {
    let out = hmn()
        .arg("validate")
        .arg(fixtures().join("with_imports/main.hmn"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stdout.is_empty());
    assert!(out.stderr.is_empty());
}

// ── compile ──

#[test]
fn compile_default() {
    let out = hmn()
        .arg("compile")
        .arg(fixtures().join("valid.hmn"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("# testbot"), "stdout: {stdout}");
    assert!(stdout.contains("NEVER: lie"), "stdout: {stdout}");
}

#[test]
fn compile_json() {
    let out = hmn()
        .arg("compile")
        .arg("-f")
        .arg("json")
        .arg(fixtures().join("valid.hmn"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON");
    assert_eq!(v["name"], "testbot");
}

#[test]
fn compile_hmn() {
    let out = hmn()
        .arg("compile")
        .arg("-f")
        .arg("hmn")
        .arg(fixtures().join("valid.hmn"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("AGENT testbot"), "stdout: {stdout}");
    assert!(stdout.contains("CONSTRAINTS safety"), "stdout: {stdout}");
    assert!(!stdout.contains("IMPORT"), "hmn output should not have imports");
}

#[test]
fn compile_all_formats() {
    for format in &["prompt", "json", "yaml", "toml", "txt", "hmn"] {
        let out = hmn()
            .arg("compile")
            .arg("-f")
            .arg(format)
            .arg(fixtures().join("valid.hmn"))
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(0),
            "format {format} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            !out.stdout.is_empty(),
            "format {format} produced no output"
        );
    }
}

#[test]
fn compile_broken() {
    let out = hmn()
        .arg("compile")
        .arg(fixtures().join("broken.hmn"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("error:"), "stderr: {stderr}");
    assert!(stderr.contains(" --> "), "should have location arrow: {stderr}");
}

#[test]
fn compile_bad_format() {
    let out = hmn()
        .arg("compile")
        .arg("-f")
        .arg("bogus")
        .arg(fixtures().join("valid.hmn"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("unknown format: bogus"), "stderr: {stderr}");
}

#[test]
fn compile_no_file() {
    let out = hmn().arg("compile").output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("usage:"), "stderr: {stderr}");
}

#[test]
fn compile_help() {
    let out = hmn().arg("compile").arg("--help").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("format"), "stderr: {stderr}");
}

// ── fmt ──

#[test]
fn fmt_stdout() {
    let out = hmn()
        .arg("fmt")
        .arg(fixtures().join("valid.hmn"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("AGENT testbot"), "stdout: {stdout}");
    assert!(stdout.contains("CONSTRAINTS safety"), "stdout: {stdout}");
}

#[test]
fn fmt_write() {
    let tmp = tempfile::TempDir::new().unwrap();
    let src = "AGENT   bot\n\nCONSTRAINTS   safety\n  NEVER   lie\n";
    let path = tmp.path().join("test.hmn");
    std::fs::write(&path, src).unwrap();

    let out = hmn()
        .arg("fmt")
        .arg("-w")
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stdout.is_empty(), "stdout should be empty with -w");

    let written = std::fs::read_to_string(&path).unwrap();
    assert!(written.contains("AGENT bot\n"), "written: {written}");
    assert!(written.contains("CONSTRAINTS safety\n"), "written: {written}");
}

#[test]
fn fmt_fragment() {
    let out = hmn()
        .arg("fmt")
        .arg(fixtures().join("fragment.hmn"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("CONSTRAINTS safety"), "stdout: {stdout}");
    assert!(stdout.contains("FLOW greet"), "stdout: {stdout}");
    assert!(!stdout.contains("AGENT"), "fragment should not have AGENT");
}

#[test]
fn fmt_broken() {
    let out = hmn()
        .arg("fmt")
        .arg(fixtures().join("broken.hmn"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("error:"), "stderr: {stderr}");
    assert!(stderr.contains(" --> "), "should have location arrow: {stderr}");
    assert!(stderr.contains(" | "), "should have gutter: {stderr}");
}

// ── additional coverage ──

#[test]
fn compile_dash_f_alone() {
    let out = hmn()
        .arg("compile")
        .arg("-f")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "bare -f should be usage error");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("usage:"), "stderr: {stderr}");
}

#[test]
fn compile_with_imports() {
    let out = hmn()
        .arg("compile")
        .arg("-f")
        .arg("json")
        .arg(fixtures().join("with_imports/main.hmn"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON");
    assert_eq!(v["name"], "testbot");
    assert!(v["constraints"].as_object().unwrap().contains_key("safety"), "should merge imported constraints");
}

#[test]
fn fmt_with_imports() {
    let out = hmn()
        .arg("fmt")
        .arg(fixtures().join("with_imports/main.hmn"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("IMPORT ./rules.hmn"), "imports should be preserved: {stdout}");
    assert!(stdout.contains("AGENT testbot"), "stdout: {stdout}");
}

#[test]
fn fmt_write_broken() {
    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path().join("bad.hmn");
    std::fs::write(&path, "NOT VALID SYNTAX\n").unwrap();

    let out = hmn()
        .arg("fmt")
        .arg("-w")
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let content = std::fs::read_to_string(&path).unwrap();
    assert_eq!(content, "NOT VALID SYNTAX\n", "broken file should not be overwritten");
}

#[test]
fn validate_help() {
    let out = hmn()
        .arg("validate")
        .arg("--help")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("usage:"), "stderr: {stderr}");
    assert!(stderr.contains("validate"), "stderr: {stderr}");
}

#[test]
fn fmt_help() {
    let out = hmn()
        .arg("fmt")
        .arg("--help")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("usage:"), "stderr: {stderr}");
    assert!(stderr.contains("fmt"), "stderr: {stderr}");
}

// ── error formatter output ──

#[test]
fn error_shows_source_context() {
    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path().join("test.hmn");
    std::fs::write(&path, "AGENT @bot\n").unwrap();

    let out = hmn()
        .arg("fmt")
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("error:"), "stderr: {stderr}");
    assert!(stderr.contains("AGENT @bot"), "should show source line: {stderr}");
    assert!(stderr.contains("^"), "should show caret: {stderr}");
}

#[test]
fn error_shows_arrow_location() {
    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path().join("test.hmn");
    std::fs::write(&path, "CONSTRAINTS\n  MUST be helpful\n").unwrap();

    let out = hmn()
        .arg("validate")
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains(" --> "), "should have --> arrow: {stderr}");
    assert!(stderr.contains("test.hmn:"), "should show filename: {stderr}");
}

#[test]
fn error_file_level_no_source() {
    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path().join("test.hmn");
    std::fs::write(&path, "CONSTRAINTS safety\n  NEVER lie\n").unwrap();

    let out = hmn()
        .arg("validate")
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("error: no AGENT declaration found"), "stderr: {stderr}");
    assert!(stderr.contains(" --> "), "should have --> arrow: {stderr}");
}

#[test]
fn multiple_resolver_errors_separated() {
    let tmp = tempfile::TempDir::new().unwrap();
    let main_path = tmp.path().join("main.hmn");
    let frag_path = tmp.path().join("frag.hmn");
    // Both files define CONSTRAINTS safety -> duplicate block error
    // frag.hmn also has AGENT -> AGENT in non-root error
    std::fs::write(&frag_path, "AGENT other\n\nCONSTRAINTS safety\n  MUST be kind\n").unwrap();
    std::fs::write(&main_path, "IMPORT ./frag.hmn\n\nAGENT bot\n\nCONSTRAINTS safety\n  NEVER lie\n").unwrap();

    let out = hmn()
        .arg("validate")
        .arg(&main_path)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    let error_blocks: Vec<&str> = stderr.split("\n\n").filter(|s| s.contains("error:")).collect();
    assert!(error_blocks.len() >= 2, "should have multiple errors separated by blank lines: {stderr}");
}
