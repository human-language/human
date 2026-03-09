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
        TokenKind::Package("safety"),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}

#[test]
fn keyword_import_package_subpath() {
    assert_eq!(lex("IMPORT safety/strict\n"), vec![
        TokenKind::Keyword(Keyword::Import),
        TokenKind::Package("safety/strict"),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}

#[test]
fn keyword_import_package_with_dots() {
    assert_eq!(lex("IMPORT org.safety.v2\n"), vec![
        TokenKind::Keyword(Keyword::Import),
        TokenKind::Package("org.safety.v2"),
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
fn ascii_error_bails_before_tokenizing() {
    // Single non-ASCII byte: should error and not produce any tokens
    let input = b"AGENT \xFF support\n";
    let lexer = Lexer::new(input);
    let result = lexer.tokenize();
    assert!(result.is_err());
    let errs = result.unwrap_err();
    assert!(errs.iter().any(|e| e.message.contains("ASCII")));
}

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

// --- Span column accuracy ---

#[test]
fn span_col_is_start_of_token() {
    // "AGENT support\n"
    //  ^     ^
    //  col1  col7
    let lexer = Lexer::new(b"AGENT support\n");
    let tokens = lexer.tokenize().unwrap();
    // tokens: [Keyword(Agent), Ident("support"), Newline, Eof]
    assert_eq!(tokens[0].span.col, 1, "AGENT should start at col 1");
    assert_eq!(tokens[1].span.col, 7, "support should start at col 7");
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

// --- Bug #6: SYSTEM path handling (rest-of-line capture) ---

#[test]
fn system_path_with_spaces() {
    assert_eq!(lex("SYSTEM ./prompts/my support file.md\n"), vec![
        TokenKind::Keyword(Keyword::System),
        TokenKind::Path("./prompts/my support file.md"),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}

#[test]
fn system_parent_path() {
    assert_eq!(lex("SYSTEM ../shared/prompts/base.md\n"), vec![
        TokenKind::Keyword(Keyword::System),
        TokenKind::Path("../shared/prompts/base.md"),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}

#[test]
fn system_no_target() {
    assert_eq!(lex("SYSTEM\n"), vec![
        TokenKind::Keyword(Keyword::System),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}

// --- Bug #1: EOF line number ---

#[test]
fn eof_line_without_trailing_newline() {
    let lexer = Lexer::new(b"AGENT support");
    let tokens = lexer.tokenize().unwrap();
    let eof = tokens.last().unwrap();
    assert_eq!(eof.span.line, 1, "EOF should be on line 1 when no trailing newline");
}

#[test]
fn eof_line_with_trailing_newline() {
    let lexer = Lexer::new(b"AGENT support\n");
    let tokens = lexer.tokenize().unwrap();
    let eof = tokens.last().unwrap();
    assert_eq!(eof.span.line, 2, "EOF should be on line 2 after trailing newline");
}

// --- Bug fix: \\r in strings ---

#[test]
fn string_unterminated_at_crlf() {
    let input = b"  INPUT \"hello\r\nworld\"\n";
    let lexer = Lexer::new(input);
    let result = lexer.tokenize();
    assert!(result.is_err());
    let errs = result.unwrap_err();
    assert!(errs.iter().any(|e| e.message.contains("unterminated string")));
}

// --- Deep indentation ---

#[test]
fn indent_deep_nest_and_unwind() {
    let input = "A\n  B\n    C\n  D\nE\n";
    let tokens = lex(input);
    let indent_count = tokens.iter().filter(|t| matches!(t, TokenKind::Indent)).count();
    let dedent_count = tokens.iter().filter(|t| matches!(t, TokenKind::Dedent)).count();
    assert_eq!(indent_count, dedent_count);
    assert_eq!(indent_count, 2);
}

#[test]
fn indent_multi_dedent_at_once() {
    let input = "A\n  B\n    C\nD\n";
    let tokens = lex(input);
    let indent_count = tokens.iter().filter(|t| matches!(t, TokenKind::Indent)).count();
    let dedent_count = tokens.iter().filter(|t| matches!(t, TokenKind::Dedent)).count();
    assert_eq!(indent_count, 2);
    assert_eq!(dedent_count, 2);
}

// --- Indentation errors ---

#[test]
fn indent_odd_spaces() {
    let errs = lex_err("   NEVER share data\n");
    assert!(errs.iter().any(|e| e.contains("multiple of 2")));
}

#[test]
fn indent_mismatch_level() {
    let errs = lex_err("CONSTRAINTS rules\n    NEVER leak\n  MUST help\n");
    assert!(errs.iter().any(|e| e.contains("does not match any outer level")));
}

// --- Blank lines ---

#[test]
fn consecutive_blank_lines() {
    let tokens = lex("\n\n\nAGENT x\n");
    assert!(tokens.contains(&TokenKind::Keyword(Keyword::Agent)));
    assert!(tokens.contains(&TokenKind::Ident("x")));
    let non_meta: Vec<_> = tokens.iter().filter(|t| !matches!(t, TokenKind::Newline | TokenKind::Eof)).collect();
    assert_eq!(non_meta.len(), 2);
}

// --- CRLF full file ---

#[test]
fn crlf_full_file() {
    let input = "AGENT support\r\nSYSTEM ./prompt.md\r\nCONSTRAINTS rules\r\n  NEVER leak\r\n";
    let tokens = lex(input);
    assert!(tokens.contains(&TokenKind::Keyword(Keyword::Agent)));
    assert!(tokens.contains(&TokenKind::Ident("support")));
    assert!(tokens.contains(&TokenKind::Keyword(Keyword::System)));
    assert!(tokens.contains(&TokenKind::Path("./prompt.md")));
    assert!(tokens.contains(&TokenKind::Keyword(Keyword::Never)));
    assert!(tokens.contains(&TokenKind::Text("leak")));
    let indent_count = tokens.iter().filter(|t| matches!(t, TokenKind::Indent)).count();
    let dedent_count = tokens.iter().filter(|t| matches!(t, TokenKind::Dedent)).count();
    assert_eq!(indent_count, dedent_count);
}

// --- Empty string literal ---

#[test]
fn string_empty() {
    assert_eq!(lex("  INPUT \"\"\n"), vec![
        TokenKind::Indent,
        TokenKind::Keyword(Keyword::Input),
        TokenKind::Str("".to_string()),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

// --- Comment with no space after # ---

#[test]
fn comment_no_space_after_hash() {
    assert_eq!(lex("#comment\n"), vec![
        TokenKind::Comment("comment"),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}

// --- Constraint at column 0 ---

#[test]
fn constraint_at_column_zero() {
    assert_eq!(lex("NEVER share data\n"), vec![
        TokenKind::Keyword(Keyword::Never),
        TokenKind::Text("share data"),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}

// --- IMPORT with no target ---

#[test]
fn import_no_target() {
    assert_eq!(lex("IMPORT\n"), vec![
        TokenKind::Keyword(Keyword::Import),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}

// --- Error accumulation cap ---

#[test]
fn error_cap_at_10() {
    let mut input = String::new();
    for _ in 0..15 {
        input.push_str("\t\tNEVER leak\n");
    }
    let errs = lex_err(&input);
    assert_eq!(errs.len(), 10);
}
