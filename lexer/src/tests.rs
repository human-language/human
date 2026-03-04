use crate::token::{TokenKind, Keyword};
use crate::lexer::Lexer;

fn lex<'a>(input: &'a str) -> Vec<TokenKind<'a>> {
    let lexer = Lexer::new(input.as_bytes());
    lexer.tokenize().unwrap().into_iter().map(|t| t.kind).collect()
}

fn lex_err(input: &str) -> Vec<String> {
    let lexer = Lexer::new(input.as_bytes());
    match lexer.tokenize() {
        Err(errs) => errs.into_iter().map(|e| e.message).collect(),
        Ok(_) => vec![],
    }
}

// --- Keywords ---

#[test]
fn keyword_agent() {
    assert_eq!(lex("AGENT support\n"), vec![
        TokenKind::Keyword(Keyword::Agent),
        TokenKind::Ident("support"),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}

#[test]
fn keyword_constraints() {
    assert_eq!(lex("CONSTRAINTS safety_rules\n"), vec![
        TokenKind::Keyword(Keyword::Constraints),
        TokenKind::Ident("safety_rules"),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}

#[test]
fn keyword_test() {
    assert_eq!(lex("TEST\n"), vec![
        TokenKind::Keyword(Keyword::Test),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}

#[test]
fn keyword_flow() {
    assert_eq!(lex("FLOW handle_request\n"), vec![
        TokenKind::Keyword(Keyword::Flow),
        TokenKind::Ident("handle_request"),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}

#[test]
fn keyword_system() {
    assert_eq!(lex("SYSTEM ./prompts/support.md\n"), vec![
        TokenKind::Keyword(Keyword::System),
        TokenKind::Path("./prompts/support.md"),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}

#[test]
fn keyword_import_path() {
    assert_eq!(lex("IMPORT ./constraints/safety.hmn\n"), vec![
        TokenKind::Keyword(Keyword::Import),
        TokenKind::Path("./constraints/safety.hmn"),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}

#[test]
fn keyword_import_package() {
    assert_eq!(lex("IMPORT safety\n"), vec![
        TokenKind::Keyword(Keyword::Import),
        TokenKind::Ident("safety"),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}

#[test]
fn keyword_import_package_subpath() {
    assert_eq!(lex("IMPORT safety/strict\n"), vec![
        TokenKind::Keyword(Keyword::Import),
        TokenKind::Ident("safety/strict"),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}

#[test]
fn keyword_input() {
    assert_eq!(lex("  INPUT \"hello\"\n"), vec![
        TokenKind::Indent,
        TokenKind::Keyword(Keyword::Input),
        TokenKind::Str("hello".to_string()),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

#[test]
fn keyword_expect_contains() {
    assert_eq!(lex("  EXPECT CONTAINS \"ticket\"\n"), vec![
        TokenKind::Indent,
        TokenKind::Keyword(Keyword::Expect),
        TokenKind::Keyword(Keyword::Contains),
        TokenKind::Str("ticket".to_string()),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

#[test]
fn keyword_expect_not_contains() {
    assert_eq!(lex("  EXPECT NOT CONTAINS \"password\"\n"), vec![
        TokenKind::Indent,
        TokenKind::Keyword(Keyword::Expect),
        TokenKind::Keyword(Keyword::Not),
        TokenKind::Keyword(Keyword::Contains),
        TokenKind::Str("password".to_string()),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

#[test]
fn keyword_expect_matches() {
    assert_eq!(lex("  EXPECT MATCHES \"REF-[0-9]+\"\n"), vec![
        TokenKind::Indent,
        TokenKind::Keyword(Keyword::Expect),
        TokenKind::Keyword(Keyword::Matches),
        TokenKind::Str("REF-[0-9]+".to_string()),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

// --- Modal text capture ---

#[test]
fn modal_never() {
    assert_eq!(lex("  NEVER share customer data\n"), vec![
        TokenKind::Indent,
        TokenKind::Keyword(Keyword::Never),
        TokenKind::Text("share customer data"),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

#[test]
fn modal_must() {
    assert_eq!(lex("  MUST respond within 30 seconds\n"), vec![
        TokenKind::Indent,
        TokenKind::Keyword(Keyword::Must),
        TokenKind::Text("respond within 30 seconds"),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

#[test]
fn modal_should() {
    assert_eq!(lex("  SHOULD maintain >95% accuracy\n"), vec![
        TokenKind::Indent,
        TokenKind::Keyword(Keyword::Should),
        TokenKind::Text("maintain >95% accuracy"),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

#[test]
fn modal_avoid() {
    assert_eq!(lex("  AVOID technical jargon\n"), vec![
        TokenKind::Indent,
        TokenKind::Keyword(Keyword::Avoid),
        TokenKind::Text("technical jargon"),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

#[test]
fn modal_may() {
    assert_eq!(lex("  MAY escalate to human\n"), vec![
        TokenKind::Indent,
        TokenKind::Keyword(Keyword::May),
        TokenKind::Text("escalate to human"),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

#[test]
fn modal_text_with_hash() {
    assert_eq!(lex("  MUST create ticket in #SUP format\n"), vec![
        TokenKind::Indent,
        TokenKind::Keyword(Keyword::Must),
        TokenKind::Text("create ticket in #SUP format"),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

#[test]
fn modal_text_with_dollar() {
    assert_eq!(lex("  MUST use $USD for all prices\n"), vec![
        TokenKind::Indent,
        TokenKind::Keyword(Keyword::Must),
        TokenKind::Text("use $USD for all prices"),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

#[test]
fn modal_keyword_in_text() {
    assert_eq!(lex("  NEVER MUST do two things at once\n"), vec![
        TokenKind::Indent,
        TokenKind::Keyword(Keyword::Never),
        TokenKind::Text("MUST do two things at once"),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

#[test]
fn modal_empty_no_text() {
    assert_eq!(lex("  NEVER\n"), vec![
        TokenKind::Indent,
        TokenKind::Keyword(Keyword::Never),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

// --- Comments ---

#[test]
fn comment_at_line_start() {
    assert_eq!(lex("# safety rules\n"), vec![
        TokenKind::Comment("safety rules"),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}

#[test]
fn comment_indented() {
    assert_eq!(lex("  # hard stops\n"), vec![
        TokenKind::Indent,
        TokenKind::Comment("hard stops"),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

// --- String literals ---

#[test]
fn string_basic() {
    assert_eq!(lex("  INPUT \"hello world\"\n"), vec![
        TokenKind::Indent,
        TokenKind::Keyword(Keyword::Input),
        TokenKind::Str("hello world".to_string()),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

#[test]
fn string_escaped_quote() {
    assert_eq!(lex("  INPUT \"say \\\"hi\\\"\"\n"), vec![
        TokenKind::Indent,
        TokenKind::Keyword(Keyword::Input),
        TokenKind::Str("say \"hi\"".to_string()),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

#[test]
fn string_escaped_backslash() {
    assert_eq!(lex("  INPUT \"path\\\\here\"\n"), vec![
        TokenKind::Indent,
        TokenKind::Keyword(Keyword::Input),
        TokenKind::Str("path\\here".to_string()),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

#[test]
fn string_unterminated() {
    let errs = lex_err("  INPUT \"hello\n");
    assert!(errs.iter().any(|e| e.contains("unterminated string")));
}

// --- Indentation ---

#[test]
fn indent_basic() {
    assert_eq!(lex("CONSTRAINTS safety\n  NEVER share data\n"), vec![
        TokenKind::Keyword(Keyword::Constraints),
        TokenKind::Ident("safety"),
        TokenKind::Newline,
        TokenKind::Indent,
        TokenKind::Keyword(Keyword::Never),
        TokenKind::Text("share data"),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

#[test]
fn indent_multi_level() {
    // 0 -> 2 -> 0 should emit INDENT then DEDENT
    let input = "AGENT support\n  SYSTEM ./p.md\nFLOW x\n";
    let tokens = lex(input);
    let indent_count = tokens.iter().filter(|t| matches!(t, TokenKind::Indent)).count();
    let dedent_count = tokens.iter().filter(|t| matches!(t, TokenKind::Dedent)).count();
    assert_eq!(indent_count, dedent_count);
}

#[test]
fn indent_blank_lines_ignored() {
    let input = "CONSTRAINTS safety\n\n  NEVER share data\n";
    let tokens = lex(input);
    assert!(tokens.contains(&TokenKind::Indent));
    assert!(tokens.contains(&TokenKind::Dedent));
}

#[test]
fn indent_eof_drain() {
    let input = "CONSTRAINTS safety\n  NEVER share data\n  MUST be helpful\n";
    let tokens = lex(input);
    let indent_count = tokens.iter().filter(|t| matches!(t, TokenKind::Indent)).count();
    let dedent_count = tokens.iter().filter(|t| matches!(t, TokenKind::Dedent)).count();
    assert_eq!(indent_count, dedent_count);
}

// --- File paths ---

#[test]
fn path_relative() {
    assert_eq!(lex("SYSTEM ./prompts/support.md\n"), vec![
        TokenKind::Keyword(Keyword::System),
        TokenKind::Path("./prompts/support.md"),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}

#[test]
fn path_parent() {
    assert_eq!(lex("IMPORT ../shared/common.hmn\n"), vec![
        TokenKind::Keyword(Keyword::Import),
        TokenKind::Path("../shared/common.hmn"),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}

// --- Numbers and booleans ---

#[test]
fn number_integer() {
    assert_eq!(lex("  max_retries = 3\n"), vec![
        TokenKind::Indent,
        TokenKind::Ident("max_retries"),
        TokenKind::Equals,
        TokenKind::Number(3.0),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

#[test]
fn number_float() {
    assert_eq!(lex("  threshold = 0.5\n"), vec![
        TokenKind::Indent,
        TokenKind::Ident("threshold"),
        TokenKind::Equals,
        TokenKind::Number(0.5),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

#[test]
fn number_negative() {
    assert_eq!(lex("  offset = -1\n"), vec![
        TokenKind::Indent,
        TokenKind::Ident("offset"),
        TokenKind::Equals,
        TokenKind::Number(-1.0),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

#[test]
fn boolean_true() {
    assert_eq!(lex("  verbose = true\n"), vec![
        TokenKind::Indent,
        TokenKind::Ident("verbose"),
        TokenKind::Equals,
        TokenKind::Bool(true),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

#[test]
fn boolean_false() {
    assert_eq!(lex("  debug = false\n"), vec![
        TokenKind::Indent,
        TokenKind::Ident("debug"),
        TokenKind::Equals,
        TokenKind::Bool(false),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

// --- Identifier vs keyword ---

#[test]
fn lowercase_not_keyword() {
    assert_eq!(lex("agent\n"), vec![
        TokenKind::Ident("agent"),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}

// --- ASCII enforcement ---

#[test]
fn ascii_reject_emoji() {
    let input = b"AGENT support \xF0\x9F\x91\x8B\n";
    let lexer = Lexer::new(input);
    assert!(lexer.tokenize().is_err());
}

#[test]
fn ascii_reject_bom() {
    let input = b"\xEF\xBB\xBFAGENT support\n";
    let lexer = Lexer::new(input);
    assert!(lexer.tokenize().is_err());
}

// --- Tab rejection ---

#[test]
fn tab_as_indentation() {
    let errs = lex_err("\tNEVER share data\n");
    assert!(errs.iter().any(|e| e.contains("tabs not allowed")));
}

// --- CRLF ---

#[test]
fn crlf_handled() {
    assert_eq!(lex("AGENT support\r\n"), vec![
        TokenKind::Keyword(Keyword::Agent),
        TokenKind::Ident("support"),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}

// --- Edge cases ---

#[test]
fn empty_file() {
    assert_eq!(lex(""), vec![
        TokenKind::Eof,
    ]);
}

#[test]
fn only_comments() {
    assert_eq!(lex("# comment one\n# comment two\n"), vec![
        TokenKind::Comment("comment one"),
        TokenKind::Newline,
        TokenKind::Comment("comment two"),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}

#[test]
fn eof_without_trailing_newline() {
    assert_eq!(lex("AGENT support"), vec![
        TokenKind::Keyword(Keyword::Agent),
        TokenKind::Ident("support"),
        TokenKind::Eof,
    ]);
}

// --- Full file: guidelines.hmn ---

#[test]
fn full_guidelines_hmn() {
    let input = "\
IMPORT ./knowledgebase.hmn

AGENT dennis

SYSTEM ./dennis.md

CONSTRAINTS mentorship_discipline
  # hard stops
  NEVER skip phases
  NEVER accept vague problem statements

CONSTRAINTS interaction_quality
  NEVER produce caricature of ritchie
  MUST be terse and direct

FLOW mentorship_session
  assess existing work
  identify current phase
";

    let tokens = lex(input);

    // Verify key structural tokens
    assert!(tokens.contains(&TokenKind::Keyword(Keyword::Import)));
    assert!(tokens.contains(&TokenKind::Path("./knowledgebase.hmn")));
    assert!(tokens.contains(&TokenKind::Keyword(Keyword::Agent)));
    assert!(tokens.contains(&TokenKind::Ident("dennis")));
    assert!(tokens.contains(&TokenKind::Keyword(Keyword::System)));
    assert!(tokens.contains(&TokenKind::Path("./dennis.md")));
    assert!(tokens.contains(&TokenKind::Keyword(Keyword::Constraints)));
    assert!(tokens.contains(&TokenKind::Ident("mentorship_discipline")));
    assert!(tokens.contains(&TokenKind::Comment("hard stops")));
    assert!(tokens.contains(&TokenKind::Keyword(Keyword::Never)));
    assert!(tokens.contains(&TokenKind::Text("skip phases")));
    assert!(tokens.contains(&TokenKind::Text("accept vague problem statements")));
    assert!(tokens.contains(&TokenKind::Keyword(Keyword::Must)));
    assert!(tokens.contains(&TokenKind::Text("be terse and direct")));
    assert!(tokens.contains(&TokenKind::Keyword(Keyword::Flow)));
    assert!(tokens.contains(&TokenKind::Ident("mentorship_session")));

    // INDENT/DEDENT balance
    let indent_count = tokens.iter().filter(|t| matches!(t, TokenKind::Indent)).count();
    let dedent_count = tokens.iter().filter(|t| matches!(t, TokenKind::Dedent)).count();
    assert_eq!(indent_count, dedent_count);

    // Ends with Eof
    assert_eq!(tokens.last(), Some(&TokenKind::Eof));
}

// --- FLOW body lines tokenize normally (not modal) ---

#[test]
fn flow_body_tokenizes_normally() {
    let input = "FLOW process\n  validate input\n  check permissions\n";
    let tokens = lex(input);
    assert!(tokens.contains(&TokenKind::Keyword(Keyword::Flow)));
    assert!(tokens.contains(&TokenKind::Ident("process")));
    assert!(tokens.contains(&TokenKind::Ident("validate")));
    assert!(tokens.contains(&TokenKind::Ident("input")));
    assert!(tokens.contains(&TokenKind::Ident("check")));
    assert!(tokens.contains(&TokenKind::Ident("permissions")));
}

// --- Property assignment ---

#[test]
fn property_assignment_string() {
    assert_eq!(lex("  model = \"gpt-4\"\n"), vec![
        TokenKind::Indent,
        TokenKind::Ident("model"),
        TokenKind::Equals,
        TokenKind::Str("gpt-4".to_string()),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}
