use std::fs;
use std::path::Path;
use tempfile::TempDir;
use crate::resolve;

fn write_file(dir: &Path, name: &str, content: &str) {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
}

fn resolve_ok(dir: &Path, root_name: &str) -> crate::types::ResolvedFile {
    let root = dir.join(root_name);
    resolve(&root, dir).expect("expected resolution to succeed")
}

fn resolve_err(dir: &Path, root_name: &str) -> Vec<crate::error::ResolveError> {
    let root = dir.join(root_name);
    resolve(&root, dir).expect_err("expected resolution to fail")
}

// ---------------------------------------------------------------------------
// Happy path tests
// ---------------------------------------------------------------------------

#[test]
fn single_file_no_imports() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "main.hmn", "\
AGENT assistant
  model = \"gpt-4\"

CONSTRAINTS safety
  NEVER produce harmful content

FLOW onboard
  greet the user

TEST
  INPUT \"hello\"
  EXPECT CONTAINS \"hello\"
");
    let resolved = resolve_ok(tmp.path(), "main.hmn");
    assert_eq!(resolved.agent.name, "assistant");
    assert_eq!(resolved.constraints.len(), 1);
    assert_eq!(resolved.flows.len(), 1);
    assert_eq!(resolved.tests.len(), 1);
    assert_eq!(resolved.sources.len(), 1);
}

#[test]
fn two_files_one_import() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "safety.hmn", "\
CONSTRAINTS safety
  NEVER produce harmful content
  MUST be respectful
");
    write_file(tmp.path(), "main.hmn", "\
IMPORT ./safety.hmn

AGENT assistant
  model = \"gpt-4\"

CONSTRAINTS tone
  SHOULD be friendly
");
    let resolved = resolve_ok(tmp.path(), "main.hmn");
    assert_eq!(resolved.agent.name, "assistant");
    assert_eq!(resolved.constraints.len(), 2);
    assert_eq!(resolved.constraints[0].name, "safety");
    assert_eq!(resolved.constraints[1].name, "tone");
    assert_eq!(resolved.sources.len(), 2);
}

#[test]
fn three_file_chain() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "c.hmn", "\
CONSTRAINTS base
  NEVER lie
");
    write_file(tmp.path(), "b.hmn", "\
IMPORT ./c.hmn

CONSTRAINTS middle
  SHOULD be concise
");
    write_file(tmp.path(), "main.hmn", "\
IMPORT ./b.hmn

AGENT assistant
  model = \"gpt-4\"

CONSTRAINTS top
  MUST be helpful
");
    let resolved = resolve_ok(tmp.path(), "main.hmn");
    assert_eq!(resolved.constraints.len(), 3);
    assert_eq!(resolved.constraints[0].name, "base");
    assert_eq!(resolved.constraints[1].name, "middle");
    assert_eq!(resolved.constraints[2].name, "top");
}

#[test]
fn diamond_dependency() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "d.hmn", "\
CONSTRAINTS shared
  NEVER be rude
");
    write_file(tmp.path(), "b.hmn", "\
IMPORT ./d.hmn

CONSTRAINTS from_b
  SHOULD be brief
");
    write_file(tmp.path(), "c.hmn", "\
IMPORT ./d.hmn

CONSTRAINTS from_c
  MUST be accurate
");
    write_file(tmp.path(), "main.hmn", "\
IMPORT ./b.hmn
IMPORT ./c.hmn

AGENT assistant
  model = \"gpt-4\"
");
    let resolved = resolve_ok(tmp.path(), "main.hmn");
    assert_eq!(resolved.constraints.len(), 3);
    assert_eq!(resolved.constraints[0].name, "shared");
    let constraint_names: Vec<&str> = resolved.constraints.iter().map(|c| c.name.as_str()).collect();
    assert!(constraint_names.contains(&"from_b"));
    assert!(constraint_names.contains(&"from_c"));
    assert_eq!(resolved.sources.len(), 4);
}

#[test]
fn package_import() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "human_modules/safety/main.hmn", "\
CONSTRAINTS pkg_safety
  NEVER produce harmful content
");
    write_file(tmp.path(), "main.hmn", "\
IMPORT safety

AGENT assistant
  model = \"gpt-4\"
");
    let resolved = resolve_ok(tmp.path(), "main.hmn");
    assert_eq!(resolved.constraints.len(), 1);
    assert_eq!(resolved.constraints[0].name, "pkg_safety");
}

#[test]
fn package_subpath_import() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "human_modules/safety/strict.hmn", "\
CONSTRAINTS strict_safety
  NEVER produce harmful content
  NEVER hallucinate
");
    write_file(tmp.path(), "main.hmn", "\
IMPORT safety/strict

AGENT assistant
  model = \"gpt-4\"
");
    let resolved = resolve_ok(tmp.path(), "main.hmn");
    assert_eq!(resolved.constraints.len(), 1);
    assert_eq!(resolved.constraints[0].name, "strict_safety");
}

#[test]
fn parent_path_import() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "shared/common.hmn", "\
CONSTRAINTS common
  MUST be helpful
");
    write_file(tmp.path(), "agents/main.hmn", "\
IMPORT ../shared/common.hmn

AGENT assistant
  model = \"gpt-4\"
");
    let resolved = resolve_ok(tmp.path(), "agents/main.hmn");
    assert_eq!(resolved.constraints.len(), 1);
    assert_eq!(resolved.constraints[0].name, "common");
}

#[test]
fn merge_order_imports_before_root() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "imported.hmn", "\
FLOW setup
  initialize the system
");
    write_file(tmp.path(), "main.hmn", "\
IMPORT ./imported.hmn

AGENT assistant
  model = \"gpt-4\"

FLOW main_flow
  handle user request
");
    let resolved = resolve_ok(tmp.path(), "main.hmn");
    assert_eq!(resolved.flows.len(), 2);
    assert_eq!(resolved.flows[0].name, "setup");
    assert_eq!(resolved.flows[1].name, "main_flow");
}

#[test]
fn sources_list_complete() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "a.hmn", "\
CONSTRAINTS from_a
  NEVER lie
");
    write_file(tmp.path(), "b.hmn", "\
CONSTRAINTS from_b
  MUST be helpful
");
    write_file(tmp.path(), "main.hmn", "\
IMPORT ./a.hmn
IMPORT ./b.hmn

AGENT assistant
  model = \"gpt-4\"
");
    let resolved = resolve_ok(tmp.path(), "main.hmn");
    assert_eq!(resolved.sources.len(), 3);
    let names: Vec<String> = resolved.sources.iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert!(names.contains(&"a.hmn".to_string()));
    assert!(names.contains(&"b.hmn".to_string()));
    assert!(names.contains(&"main.hmn".to_string()));
    assert_eq!(names.last().unwrap(), "main.hmn");
}

#[test]
fn tests_from_imports_included() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "checks.hmn", "\
TEST
  INPUT \"test input\"
  EXPECT CONTAINS \"expected\"
");
    write_file(tmp.path(), "main.hmn", "\
IMPORT ./checks.hmn

AGENT assistant
  model = \"gpt-4\"

TEST
  INPUT \"hello\"
  EXPECT CONTAINS \"hi\"
");
    let resolved = resolve_ok(tmp.path(), "main.hmn");
    assert_eq!(resolved.tests.len(), 2);
}

// ---------------------------------------------------------------------------
// Error tests
// ---------------------------------------------------------------------------

#[test]
fn circular_import() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "a.hmn", "\
IMPORT ./b.hmn

AGENT assistant
  model = \"gpt-4\"
");
    write_file(tmp.path(), "b.hmn", "\
IMPORT ./a.hmn

CONSTRAINTS safety
  NEVER lie
");
    let errs = resolve_err(tmp.path(), "a.hmn");
    let msg = errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n");
    assert!(msg.contains("circular import"), "expected circular import error, got: {}", msg);
}

#[test]
fn self_import() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "main.hmn", "\
IMPORT ./main.hmn

AGENT assistant
  model = \"gpt-4\"
");
    let errs = resolve_err(tmp.path(), "main.hmn");
    let msg = errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n");
    assert!(msg.contains("circular import"), "expected circular import error, got: {}", msg);
}

#[test]
fn transitive_cycle() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "a.hmn", "\
IMPORT ./b.hmn

AGENT assistant
  model = \"gpt-4\"
");
    write_file(tmp.path(), "b.hmn", "\
IMPORT ./c.hmn

CONSTRAINTS b_rules
  MUST be safe
");
    write_file(tmp.path(), "c.hmn", "\
IMPORT ./a.hmn

CONSTRAINTS c_rules
  NEVER lie
");
    let errs = resolve_err(tmp.path(), "a.hmn");
    let msg = errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n");
    assert!(msg.contains("circular import"), "expected circular import error, got: {}", msg);
}

#[test]
fn missing_file() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "main.hmn", "\
IMPORT ./nonexistent.hmn

AGENT assistant
  model = \"gpt-4\"
");
    let errs = resolve_err(tmp.path(), "main.hmn");
    let msg = errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n");
    assert!(msg.contains("file not found"), "expected file not found error, got: {}", msg);
    assert!(msg.contains("nonexistent.hmn"), "expected path in error, got: {}", msg);
}

#[test]
fn missing_package() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "main.hmn", "\
IMPORT nosuchpkg

AGENT assistant
  model = \"gpt-4\"
");
    let errs = resolve_err(tmp.path(), "main.hmn");
    let msg = errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n");
    assert!(msg.contains("file not found"), "expected file not found error, got: {}", msg);
    assert!(msg.contains("human_modules"), "expected human_modules in path, got: {}", msg);
}

#[test]
fn agent_in_non_root() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "imported.hmn", "\
AGENT rogue
  model = \"gpt-4\"

CONSTRAINTS imported_rules
  NEVER lie
");
    write_file(tmp.path(), "main.hmn", "\
IMPORT ./imported.hmn

AGENT assistant
  model = \"gpt-4\"
");
    let errs = resolve_err(tmp.path(), "main.hmn");
    let msg = errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n");
    assert!(msg.contains("AGENT can only appear in main.hmn"), "expected AGENT error, got: {}", msg);
}

#[test]
fn no_agent_in_root() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "main.hmn", "\
CONSTRAINTS safety
  NEVER lie
");
    let errs = resolve_err(tmp.path(), "main.hmn");
    let msg = errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n");
    assert!(msg.contains("no AGENT declaration found"), "expected no AGENT error, got: {}", msg);
}

#[test]
fn duplicate_constraints_name() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "imported.hmn", "\
CONSTRAINTS safety
  NEVER produce harmful content
");
    write_file(tmp.path(), "main.hmn", "\
IMPORT ./imported.hmn

AGENT assistant
  model = \"gpt-4\"

CONSTRAINTS safety
  MUST be safe
");
    let errs = resolve_err(tmp.path(), "main.hmn");
    let msg = errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n");
    assert!(msg.contains("duplicate CONSTRAINTS block 'safety'"), "expected duplicate error, got: {}", msg);
}

#[test]
fn duplicate_flow_name() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "imported.hmn", "\
FLOW onboard
  greet the user
");
    write_file(tmp.path(), "main.hmn", "\
IMPORT ./imported.hmn

AGENT assistant
  model = \"gpt-4\"

FLOW onboard
  welcome the user
");
    let errs = resolve_err(tmp.path(), "main.hmn");
    let msg = errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n");
    assert!(msg.contains("duplicate FLOW block 'onboard'"), "expected duplicate error, got: {}", msg);
}

#[test]
fn lex_error_in_import() {
    let tmp = TempDir::new().unwrap();
    let bad_bytes: &[u8] = &[0xFF, 0xFE, b' ', b'b', b'a', b'd'];
    fs::write(tmp.path().join("bad.hmn"), bad_bytes).unwrap();
    write_file(tmp.path(), "main.hmn", "\
IMPORT ./bad.hmn

AGENT assistant
  model = \"gpt-4\"
");
    let errs = resolve_err(tmp.path(), "main.hmn");
    let msg = errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n");
    assert!(msg.contains("bad.hmn"), "expected error to reference bad.hmn, got: {}", msg);
}

#[test]
fn parse_error_in_import() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "bad.hmn", "AGENT\n");
    write_file(tmp.path(), "main.hmn", "\
IMPORT ./bad.hmn

AGENT assistant
  model = \"gpt-4\"
");
    let errs = resolve_err(tmp.path(), "main.hmn");
    let msg = errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n");
    assert!(msg.contains("bad.hmn"), "expected error to reference bad.hmn, got: {}", msg);
}

#[test]
fn path_escape_rejected() {
    let tmp = TempDir::new().unwrap();
    let parent = tmp.path().parent().unwrap();
    write_file(parent, "escaped.hmn", "\
CONSTRAINTS escaped
  NEVER be accessed
");
    write_file(tmp.path(), "main.hmn", "\
IMPORT ../escaped.hmn

AGENT assistant
  model = \"gpt-4\"
");
    let errs = resolve_err(tmp.path(), "main.hmn");
    let msg = errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n");
    assert!(
        msg.contains("escapes project root"),
        "expected path escape error, got: {}",
        msg
    );
}

#[test]
fn root_file_not_found() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("nonexistent.hmn");
    let errs = resolve(&root, tmp.path()).expect_err("expected error for missing root");
    let msg = errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n");
    assert!(msg.contains("file not found"), "expected file not found error, got: {}", msg);
}

#[test]
fn project_root_not_found() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "main.hmn", "\
AGENT assistant
  model = \"gpt-4\"
");
    let root = tmp.path().join("main.hmn");
    let bad_project = tmp.path().join("no_such_dir");
    let errs = resolve(&root, &bad_project).expect_err("expected error for missing project root");
    let msg = errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n");
    assert!(msg.contains("project root not found"), "expected project root error, got: {}", msg);
}

#[test]
fn system_in_imported_file() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "imported.hmn", "\
SYSTEM ./prompts/bot.md
CONSTRAINTS rules
  NEVER lie
");
    write_file(tmp.path(), "main.hmn", "\
IMPORT ./imported.hmn

AGENT assistant
  model = \"gpt-4\"
");
    let errs = resolve_err(tmp.path(), "main.hmn");
    let msg = errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n");
    assert!(msg.contains("imported.hmn"), "expected error to reference imported.hmn, got: {}", msg);
    assert!(msg.contains("SYSTEM"), "expected SYSTEM-related error, got: {}", msg);
}

// ---------------------------------------------------------------------------
// Edge case tests
// ---------------------------------------------------------------------------

#[test]
fn empty_imported_file() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "empty.hmn", "# just a comment\n");
    write_file(tmp.path(), "main.hmn", "\
IMPORT ./empty.hmn

AGENT assistant
  model = \"gpt-4\"
");
    let resolved = resolve_ok(tmp.path(), "main.hmn");
    assert_eq!(resolved.agent.name, "assistant");
    assert_eq!(resolved.constraints.len(), 0);
    assert_eq!(resolved.sources.len(), 2);
}

#[test]
fn deep_chain() {
    let tmp = TempDir::new().unwrap();
    for i in (0..10).rev() {
        if i == 9 {
            write_file(tmp.path(), &format!("f{}.hmn", i), &format!("\
CONSTRAINTS rules_{i}
  NEVER break rule {i}
"));
        } else {
            write_file(tmp.path(), &format!("f{}.hmn", i), &format!("\
IMPORT ./f{next}.hmn

CONSTRAINTS rules_{i}
  NEVER break rule {i}
", next = i + 1));
        }
    }
    write_file(tmp.path(), "main.hmn", "\
IMPORT ./f0.hmn

AGENT assistant
  model = \"gpt-4\"
");
    let resolved = resolve_ok(tmp.path(), "main.hmn");
    assert_eq!(resolved.constraints.len(), 10);
    assert_eq!(resolved.constraints[0].name, "rules_9");
    assert_eq!(resolved.constraints[9].name, "rules_0");
    assert_eq!(resolved.sources.len(), 11);
}

#[test]
fn multiple_imports_same_file() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "shared.hmn", "\
CONSTRAINTS shared
  NEVER lie
");
    write_file(tmp.path(), "a.hmn", "\
IMPORT ./shared.hmn

CONSTRAINTS from_a
  SHOULD be friendly
");
    write_file(tmp.path(), "b.hmn", "\
IMPORT ./shared.hmn

CONSTRAINTS from_b
  MUST be helpful
");
    write_file(tmp.path(), "main.hmn", "\
IMPORT ./a.hmn
IMPORT ./b.hmn

AGENT assistant
  model = \"gpt-4\"
");
    let resolved = resolve_ok(tmp.path(), "main.hmn");
    let shared_count = resolved.constraints.iter()
        .filter(|c| c.name == "shared")
        .count();
    assert_eq!(shared_count, 1, "shared should appear exactly once");
    assert_eq!(resolved.constraints.len(), 3);
}
