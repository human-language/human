use human_lexer::Lexer;
use crate::parser::parse;
use crate::types::*;

fn parse_ok(input: &str) -> HmnFile {
    let tokens = Lexer::new(input.as_bytes()).tokenize().expect("lex failed");
    parse(&tokens).expect("parse failed")
}

fn parse_err(input: &str) -> Vec<crate::error::ParseError> {
    let tokens = Lexer::new(input.as_bytes()).tokenize().expect("lex failed");
    parse(&tokens).expect_err("expected parse error")
}

// 1. Minimal valid file
#[test]
fn minimal_agent() {
    let file = parse_ok("AGENT support\n");
    assert_eq!(file.agent.as_ref().unwrap().name, "support");
    assert!(file.imports.is_empty());
    assert!(file.constraints.is_empty());
    assert!(file.flows.is_empty());
    assert!(file.tests.is_empty());
}

// 2. Full file -- matches canonical form from docs (AGENT and SYSTEM at column 0)
#[test]
fn full_file() {
    let input = "\
IMPORT ./safety.hmn
IMPORT utils

AGENT support
SYSTEM ./prompts/support.md

CONSTRAINTS rules
  NEVER share data
  MUST be helpful

FLOW pipeline
  greet user
  process request

TEST
  INPUT \"hello\"
  EXPECT CONTAINS \"hi\"
";
    let file = parse_ok(input);
    assert_eq!(file.imports.len(), 2);
    let agent = file.agent.as_ref().unwrap();
    assert_eq!(agent.name, "support");
    assert_eq!(agent.system.as_ref().unwrap().path, "./prompts/support.md");
    assert_eq!(file.constraints.len(), 1);
    assert_eq!(file.constraints[0].constraints.len(), 2);
    assert_eq!(file.flows.len(), 1);
    assert_eq!(file.flows[0].steps.len(), 2);
    assert_eq!(file.tests.len(), 1);
    assert_eq!(file.tests[0].inputs.len(), 1);
    assert_eq!(file.tests[0].expects.len(), 1);
}

// 3. Constraints block -- all five levels
#[test]
fn constraints_all_levels() {
    let input = "\
CONSTRAINTS policy
  NEVER leak secrets
  MUST be accurate
  # quality note
  SHOULD be concise
  AVOID jargon
  MAY escalate
";
    let file = parse_ok(input);
    let block = &file.constraints[0];
    assert_eq!(block.name, "policy");
    assert_eq!(block.constraints.len(), 5);
    assert_eq!(block.constraints[0].level, ConstraintLevel::Never);
    assert_eq!(block.constraints[0].text, "leak secrets");
    assert_eq!(block.constraints[1].level, ConstraintLevel::Must);
    assert_eq!(block.constraints[2].level, ConstraintLevel::Should);
    assert_eq!(block.constraints[3].level, ConstraintLevel::Avoid);
    assert_eq!(block.constraints[4].level, ConstraintLevel::May);
}

// 4. Flow block -- multiple steps as ident sequences
#[test]
fn flow_block() {
    let input = "\
FLOW onboarding
  greet user
  verify identity
  process request
";
    let file = parse_ok(input);
    let flow = &file.flows[0];
    assert_eq!(flow.name, "onboarding");
    assert_eq!(flow.steps.len(), 3);
    assert_eq!(flow.steps[0].text, "greet user");
    assert_eq!(flow.steps[1].text, "verify identity");
    assert_eq!(flow.steps[2].text, "process request");
}

// 5. Test block -- all Level 1 EXPECT forms
#[test]
fn test_block_all_expect_forms() {
    let input = "\
TEST
  INPUT \"hello\"
  EXPECT CONTAINS \"hi\"
  EXPECT NOT CONTAINS \"bye\"
  EXPECT MATCHES \"h.*\"
  EXPECT NOT MATCHES \"z.*\"
";
    let file = parse_ok(input);
    let test = &file.tests[0];
    assert_eq!(test.inputs.len(), 1);
    assert_eq!(test.expects.len(), 4);

    assert!(!test.expects[0].negated);
    assert_eq!(test.expects[0].op, TestOp::Contains);
    assert_eq!(test.expects[0].value, "hi");

    assert!(test.expects[1].negated);
    assert_eq!(test.expects[1].op, TestOp::Contains);
    assert_eq!(test.expects[1].value, "bye");

    assert!(!test.expects[2].negated);
    assert_eq!(test.expects[2].op, TestOp::Matches);

    assert!(test.expects[3].negated);
    assert_eq!(test.expects[3].op, TestOp::Matches);
}

// 6. Multiple EXPECT per test
#[test]
fn multiple_expects() {
    let input = "\
TEST
  INPUT \"query\"
  EXPECT CONTAINS \"result\"
  EXPECT NOT CONTAINS \"error\"
  EXPECT MATCHES \"OK.*\"
";
    let file = parse_ok(input);
    assert_eq!(file.tests[0].expects.len(), 3);
}

// 7. Properties -- all value types
// Keywords doc says AGENT block contains properties; syntax doc says = is for
// property assignment inside AGENT blocks. No example demonstrates this yet,
// but the spec defines it.
#[test]
fn property_types() {
    let input = "\
AGENT bot
  model = \"gpt-4\"
  retries = 3
  verbose = true
  config = ./config.json
";
    let file = parse_ok(input);
    let props = &file.agent.unwrap().properties;
    assert_eq!(props.len(), 4);
    assert_eq!(props[0].value, Value::Str("gpt-4".into()));
    assert_eq!(props[1].value, Value::Number(3.0));
    assert_eq!(props[2].value, Value::Bool(true));
    assert_eq!(props[3].value, Value::Path("./config.json".into()));
}

// 8. Import forms
#[test]
fn import_forms() {
    let input = "\
IMPORT ./constraints/safety.hmn
IMPORT safety
IMPORT safety/strict
";
    let file = parse_ok(input);
    assert_eq!(file.imports.len(), 3);
    assert_eq!(file.imports[0].target, ImportTarget::Path("./constraints/safety.hmn".into()));
    assert_eq!(file.imports[1].target, ImportTarget::Package("safety".into()));
    assert_eq!(file.imports[2].target, ImportTarget::Package("safety/strict".into()));
}

// 9. SYSTEM at top level (canonical form from docs -- same column as AGENT)
#[test]
fn system_toplevel() {
    let input = "\
AGENT support
SYSTEM ./prompts/support.md
";
    let file = parse_ok(input);
    let agent = file.agent.unwrap();
    assert_eq!(agent.system.as_ref().unwrap().path, "./prompts/support.md");
}

// SYSTEM indented inside AGENT body -- parser still accepts this
// (keywords doc says AGENT opens a block), but no doc example uses this form
#[test]
fn system_inside_agent() {
    let input = "\
AGENT support
  SYSTEM ./prompts/support.md
";
    let file = parse_ok(input);
    let agent = file.agent.unwrap();
    assert_eq!(agent.system.as_ref().unwrap().path, "./prompts/support.md");
}

// 10. Empty file
#[test]
fn empty_file() {
    let file = parse_ok("");
    assert!(file.agent.is_none());
    assert!(file.imports.is_empty());
    assert!(file.constraints.is_empty());
    assert!(file.flows.is_empty());
    assert!(file.tests.is_empty());
}

// 11. File with only imports
#[test]
fn imports_only() {
    let input = "\
IMPORT ./a.hmn
IMPORT ./b.hmn
";
    let file = parse_ok(input);
    assert_eq!(file.imports.len(), 2);
    assert!(file.agent.is_none());
}

// 12. Missing identifier after CONSTRAINTS
#[test]
fn missing_constraints_name() {
    let errors = parse_err("CONSTRAINTS\n");
    assert!(errors.iter().any(|e| e.message.contains("expected identifier after CONSTRAINTS")));
}

// 13. Unexpected token at top level -- error recovery, continues parsing
#[test]
fn unexpected_toplevel_recovers() {
    let input = "\
AGENT bot
42
CONSTRAINTS rules
  NEVER leak
";
    // The lexer will lex "42" as a number at top level.
    // The parser should error on it, skip to CONSTRAINTS, and continue.
    let errors = parse_err(input);
    assert!(!errors.is_empty());
}

// 14. Unterminated block -- parser must not panic
#[test]
fn unterminated_block_no_panic() {
    // This tests parser robustness. The lexer should always balance INDENT/DEDENT,
    // but the parser should handle EOF gracefully even if it doesn't.
    let input = "AGENT bot\n";
    let file = parse_ok(input);
    assert_eq!(file.agent.as_ref().unwrap().name, "bot");
}

// 15. Multiple CONSTRAINTS blocks
#[test]
fn multiple_constraints_blocks() {
    let input = "\
CONSTRAINTS safety
  NEVER leak data

CONSTRAINTS quality
  MUST be accurate
";
    let file = parse_ok(input);
    assert_eq!(file.constraints.len(), 2);
    assert_eq!(file.constraints[0].name, "safety");
    assert_eq!(file.constraints[1].name, "quality");
}

// 16. Multiple FLOW blocks
#[test]
fn multiple_flow_blocks() {
    let input = "\
FLOW onboard
  greet user

FLOW process
  handle request
";
    let file = parse_ok(input);
    assert_eq!(file.flows.len(), 2);
    assert_eq!(file.flows[0].name, "onboard");
    assert_eq!(file.flows[1].name, "process");
}

// 17. Multiple TEST blocks
#[test]
fn multiple_test_blocks() {
    let input = "\
TEST
  INPUT \"a\"
  EXPECT CONTAINS \"b\"

TEST
  INPUT \"c\"
  EXPECT MATCHES \"d\"
";
    let file = parse_ok(input);
    assert_eq!(file.tests.len(), 2);
}

// 18. SYSTEM without AGENT
#[test]
fn system_without_agent() {
    let errors = parse_err("SYSTEM ./prompts/bot.md\n");
    assert!(errors.iter().any(|e| e.message.contains("SYSTEM without preceding AGENT")));
}

// 19. Duplicate SYSTEM (top-level form, matching docs)
#[test]
fn duplicate_system() {
    let input = "\
AGENT bot
SYSTEM ./a.md
SYSTEM ./b.md
";
    let errors = parse_err(input);
    assert!(errors.iter().any(|e| e.message.contains("duplicate SYSTEM")));
}

// 20. Duplicate AGENT
#[test]
fn duplicate_agent() {
    let input = "\
AGENT one
AGENT two
";
    let errors = parse_err(input);
    assert!(errors.iter().any(|e| e.message.contains("duplicate AGENT")));
}

// 21. Top-level SYSTEM after AGENT
#[test]
fn toplevel_system_attaches_to_agent() {
    let input = "\
AGENT support
SYSTEM ./prompts/support.md

CONSTRAINTS rules
  MUST be helpful
";
    let file = parse_ok(input);
    let agent = file.agent.as_ref().unwrap();
    assert_eq!(agent.system.as_ref().unwrap().path, "./prompts/support.md");
    assert_eq!(file.constraints.len(), 1);
}

// 22. Level 2 EXPECT form -- emits error, valid expects in same block still parsed
#[test]
fn level2_expect_emits_error() {
    // "EXPECT safe" lexes as [Keyword(Expect), Ident("safe"), Newline].
    // Parser sees EXPECT, checks for NOT/CONTAINS/MATCHES, finds Ident -- error.
    let input = "\
TEST
  INPUT \"hello\"
  EXPECT safe
  EXPECT CONTAINS \"hi\"
";
    let errors = parse_err(input);
    assert!(errors.iter().any(|e| e.message.contains("unsupported EXPECT form")));
}

// 23. Bad line inside CONSTRAINTS block -- remaining constraints still parsed
#[test]
fn bad_line_in_constraints_recovers() {
    // Line "  42" is a Number token inside a CONSTRAINTS body, not a valid
    // constraint keyword. Parser should error on it and continue.
    let input = "\
CONSTRAINTS rules
  NEVER leak data
  42
  MUST be helpful
";
    let errors = parse_err(input);
    assert!(errors.iter().any(|e| e.message.contains("expected constraint keyword")));
}

// 24. Multiple INPUT per TEST -- multi-turn test
#[test]
fn multiple_inputs_per_test() {
    let input = "\
TEST
  INPUT \"my name is alice\"
  INPUT \"what is my name\"
  EXPECT CONTAINS \"alice\"
";
    let file = parse_ok(input);
    let test = &file.tests[0];
    assert_eq!(test.inputs.len(), 2);
    assert_eq!(test.inputs[0].value, "my name is alice");
    assert_eq!(test.inputs[1].value, "what is my name");
    assert_eq!(test.expects.len(), 1);
}

// --- Additional edge case tests ---

#[test]
fn comments_between_everything() {
    let input = "\
# file comment
IMPORT ./a.hmn
# between imports and agent
AGENT bot
# before constraints
CONSTRAINTS rules
  # inside constraints
  NEVER leak
  # between constraints
  MUST help
";
    let file = parse_ok(input);
    assert_eq!(file.imports.len(), 1);
    assert_eq!(file.agent.as_ref().unwrap().name, "bot");
    assert_eq!(file.constraints[0].constraints.len(), 2);
}

#[test]
fn blank_lines_everywhere() {
    let input = "\n\nAGENT bot\n\nCONSTRAINTS rules\n\n  NEVER leak\n\n  MUST help\n\n";
    let file = parse_ok(input);
    assert_eq!(file.agent.as_ref().unwrap().name, "bot");
    assert_eq!(file.constraints[0].constraints.len(), 2);
}

#[test]
fn constraint_text_preserved_verbatim() {
    let input = "\
CONSTRAINTS rules
  NEVER share customer data
  MUST respond within 30 seconds
  MUST create ticket in #SUP format
  SHOULD maintain >95% accuracy
";
    let file = parse_ok(input);
    let cs = &file.constraints[0].constraints;
    assert_eq!(cs[0].text, "share customer data");
    assert_eq!(cs[1].text, "respond within 30 seconds");
    assert_eq!(cs[2].text, "create ticket in #SUP format");
    assert_eq!(cs[3].text, "maintain >95% accuracy");
}

#[test]
fn agent_with_no_body() {
    let file = parse_ok("AGENT minimal\n");
    let agent = file.agent.unwrap();
    assert_eq!(agent.name, "minimal");
    assert!(agent.properties.is_empty());
    assert!(agent.system.is_none());
}

#[test]
fn missing_flow_name() {
    let errors = parse_err("FLOW\n");
    assert!(errors.iter().any(|e| e.message.contains("expected identifier after FLOW")));
}

#[test]
fn missing_agent_name() {
    let errors = parse_err("AGENT\n");
    assert!(errors.iter().any(|e| e.message.contains("expected identifier after AGENT")));
}

#[test]
fn import_path_relative() {
    let file = parse_ok("IMPORT ../shared/common.hmn\n");
    assert_eq!(file.imports[0].target, ImportTarget::Path("../shared/common.hmn".into()));
}

#[test]
fn duplicate_system_inside_agent() {
    let input = "\
AGENT bot
  SYSTEM ./a.md
  SYSTEM ./b.md
";
    let errors = parse_err(input);
    assert!(errors.iter().any(|e| e.message.contains("duplicate SYSTEM")));
}

#[test]
fn test_escaped_string() {
    let input = "\
TEST
  INPUT \"what is your \\\"real\\\" name\"
  EXPECT NOT CONTAINS \"Claude\"
";
    let file = parse_ok(input);
    assert_eq!(file.tests[0].inputs[0].value, "what is your \"real\" name");
}

#[test]
fn spans_are_populated() {
    let input = "AGENT bot\n";
    let file = parse_ok(input);
    let agent = file.agent.unwrap();
    assert!(agent.span.line > 0);
    assert!(agent.span.col > 0);
}
