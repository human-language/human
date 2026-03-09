use crate::{Diagnostic, render, render_batch};

fn d(file: &str, line: u32, col: u16, len: u16, msg: &str) -> Diagnostic {
    Diagnostic {
        file: file.to_string(),
        line,
        col,
        len,
        message: msg.to_string(),
    }
}

// ── Core rendering ──

#[test]
fn single_caret() {
    let src = b"AGENT bot\nCONSTRAINTS safety\n  NEVER @lie\n";
    let diag = d("main.hmn", 3, 9, 0, "unexpected character '@'");
    let out = render(&diag, Some(src));
    assert_eq!(out, "\
error: unexpected character '@'
 --> main.hmn:3:9
  |
3 |   NEVER @lie
  |         ^
");
}

#[test]
fn underline_span() {
    let src = b"AGENT bot\n  SOMETIMES be nice\n";
    let diag = d("main.hmn", 2, 3, 9, "unknown constraint level");
    let out = render(&diag, Some(src));
    assert_eq!(out, "\
error: unknown constraint level
 --> main.hmn:2:3
  |
2 |   SOMETIMES be nice
  |   ^~~~~~~~~
");
}

#[test]
fn single_char_span() {
    let src = b"AGENT @bot\n";
    let diag = d("main.hmn", 1, 7, 1, "unexpected character '@'");
    let out = render(&diag, Some(src));
    assert_eq!(out, "\
error: unexpected character '@'
 --> main.hmn:1:7
  |
1 | AGENT @bot
  |       ^
");
}

#[test]
fn first_line_first_col() {
    let src = b"@invalid\n";
    let diag = d("test.hmn", 1, 1, 0, "unexpected character '@'");
    let out = render(&diag, Some(src));
    assert_eq!(out, "\
error: unexpected character '@'
 --> test.hmn:1:1
  |
1 | @invalid
  | ^
");
}

// ── Caret clamping ──

#[test]
fn underline_clamped_to_line_end() {
    let src = b"hi\n";
    let diag = d("test.hmn", 1, 1, 20, "bad token");
    let out = render(&diag, Some(src));
    assert_eq!(out, "\
error: bad token
 --> test.hmn:1:1
  |
1 | hi
  | ^~
");
}

#[test]
fn col_past_end_of_line() {
    let src = b"short\n";
    let diag = d("test.hmn", 1, 10, 0, "unexpected EOF");
    let out = render(&diag, Some(src));
    assert_eq!(out, "\
error: unexpected EOF
 --> test.hmn:1:10
  |
1 | short
");
}

#[test]
fn col_at_end_of_line() {
    let src = b"abcde\n";
    // col 6 = one past the last character (line is 5 chars)
    let diag = d("test.hmn", 1, 6, 0, "expected newline");
    let out = render(&diag, Some(src));
    assert_eq!(out, "\
error: expected newline
 --> test.hmn:1:6
  |
1 | abcde
");
}

// ── Fallback modes ──

#[test]
fn no_source() {
    let diag = d("main.hmn", 3, 8, 0, "unexpected token");
    let out = render(&diag, None);
    assert_eq!(out, "\
error: unexpected token
 --> main.hmn:3:8
");
}

#[test]
fn file_level_error() {
    let diag = d("rules.hmn", 0, 0, 0, "no AGENT declaration found");
    let out = render(&diag, None);
    assert_eq!(out, "\
error: no AGENT declaration found
 --> rules.hmn
");
}

#[test]
fn line_out_of_range() {
    let src = b"line1\nline2\nline3\n";
    let diag = d("test.hmn", 999, 1, 0, "gone");
    let out = render(&diag, Some(src));
    assert_eq!(out, "\
error: gone
 --> test.hmn:999:1
");
}

#[test]
fn empty_file() {
    let src = b"";
    let diag = d("test.hmn", 1, 1, 0, "empty");
    let out = render(&diag, Some(src));
    assert_eq!(out, "\
error: empty
 --> test.hmn:1:1
");
}

#[test]
fn empty_source_line() {
    let src = b"first\n\nthird\n";
    let diag = d("test.hmn", 2, 1, 0, "blank line");
    let out = render(&diag, Some(src));
    assert_eq!(out, "\
error: blank line
 --> test.hmn:2:1
  |
2 | 
");
}

// ── Line ending handling ──

#[test]
fn crlf_stripped() {
    let src = b"hello world\r\nsecond line\r\n";
    let diag = d("test.hmn", 1, 7, 5, "bad span");
    let out = render(&diag, Some(src));
    assert_eq!(out, "\
error: bad span
 --> test.hmn:1:7
  |
1 | hello world
  |       ^~~~~
");
}

#[test]
fn cr_only_stripped() {
    let src = b"aaa\rbbb\rccc";
    let diag = d("test.hmn", 2, 1, 3, "here");
    let out = render(&diag, Some(src));
    assert_eq!(out, "\
error: here
 --> test.hmn:2:1
  |
2 | bbb
  | ^~~
");
}

#[test]
fn mixed_line_endings() {
    let src = b"unix\nwindows\r\nold_mac\rfourth\n";
    let d1 = d("test.hmn", 1, 1, 4, "line 1");
    let d2 = d("test.hmn", 2, 1, 7, "line 2");
    let d3 = d("test.hmn", 3, 1, 7, "line 3");
    let d4 = d("test.hmn", 4, 1, 6, "line 4");
    let out = render_batch(&[d1, d2, d3, d4], Some(src));
    assert!(out.contains("1 | unix"), "out: {out}");
    assert!(out.contains("2 | windows"), "out: {out}");
    assert!(out.contains("3 | old_mac"), "out: {out}");
    assert!(out.contains("4 | fourth"), "out: {out}");
}

// ── Gutter formatting ──

#[test]
fn wide_gutter() {
    let src = std::iter::repeat("x\n").take(1234).collect::<String>();
    let diag = d("test.hmn", 1234, 1, 0, "deep");
    let out = render(&diag, Some(src.as_bytes()));
    assert!(out.contains("1234 | x\n"), "out: {out}");
    assert!(out.contains("     | ^\n"), "out: {out}");
}

#[test]
fn gutter_width_1() {
    let src = b"hello\n";
    let diag = d("test.hmn", 1, 1, 0, "x");
    let out = render(&diag, Some(src));
    assert!(out.contains("1 | hello\n"), "out: {out}");
    assert!(out.contains("  | ^\n"), "out: {out}");
}

#[test]
fn gutter_width_5() {
    let src = std::iter::repeat("x\n").take(99999).collect::<String>();
    let diag = d("test.hmn", 99999, 1, 0, "far");
    let out = render(&diag, Some(src.as_bytes()));
    assert!(out.contains("99999 | x\n"), "out: {out}");
    assert!(out.contains("      | ^\n"), "out: {out}");
}

// ── Batch rendering ──

#[test]
fn multiple_diagnostics() {
    let src = b"line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\nline11\nline12\n";
    let ds = vec![
        d("test.hmn", 2, 1, 5, "err one"),
        d("test.hmn", 5, 1, 5, "err two"),
        d("test.hmn", 12, 1, 6, "err three"),
    ];
    let out = render_batch(&ds, Some(src));

    // Gutter width should be 2 (widest line is 12)
    assert!(out.contains(" 2 | line2\n"), "out: {out}");
    assert!(out.contains(" 5 | line5\n"), "out: {out}");
    assert!(out.contains("12 | line12\n"), "out: {out}");

    // Blank line between diagnostics
    let parts: Vec<&str> = out.split("\n\n").collect();
    assert_eq!(parts.len(), 3, "should have 3 diagnostics separated by blank lines, got: {out}");
}

#[test]
fn single_in_batch() {
    let src = b"hello\n";
    let ds = vec![d("test.hmn", 1, 1, 0, "one")];
    let out = render_batch(&ds, Some(src));
    assert!(!out.ends_with("\n\n"), "no trailing blank line");
    assert!(out.ends_with("^\n"), "out: {out}");
}

#[test]
fn zero_diagnostics() {
    let out = render_batch(&[], Some(b"anything"));
    assert_eq!(out, "");
}

#[test]
fn gutter_uniform_in_batch() {
    let src = std::iter::repeat("x\n").take(100).collect::<String>();
    let ds = vec![
        d("test.hmn", 1, 1, 0, "first"),
        d("test.hmn", 100, 1, 0, "last"),
    ];
    let out = render_batch(&ds, Some(src.as_bytes()));
    // Gutter width 3 for both (100 has 3 digits)
    assert!(out.contains("  1 | x\n"), "out: {out}");
    assert!(out.contains("100 | x\n"), "out: {out}");
}

#[test]
fn ten_errors_batch() {
    let src = std::iter::repeat("abcdef\n").take(10).collect::<String>();
    let ds: Vec<Diagnostic> = (1..=10).map(|i| d("test.hmn", i, 1, 0, &format!("err {i}"))).collect();
    let out = render_batch(&ds, Some(src.as_bytes()));
    let parts: Vec<&str> = out.split("\n\n").collect();
    assert_eq!(parts.len(), 10, "10 diagnostics, got: {}", parts.len());
    // Gutter width 2 (widest line is 10)
    assert!(out.contains(" 1 | abcdef\n"), "out: {out}");
    assert!(out.contains("10 | abcdef\n"), "out: {out}");
}

// ── Real-world error messages ──

#[test]
fn lex_error_unexpected_char() {
    let src = b"AGENT @bot\n";
    let diag = d("main.hmn", 1, 7, 0, "unexpected character '@'");
    let out = render(&diag, Some(src));
    assert_eq!(out, "\
error: unexpected character '@'
 --> main.hmn:1:7
  |
1 | AGENT @bot
  |       ^
");
}

#[test]
fn lex_error_unterminated_string() {
    let src = b"name = \"hello\n";
    let diag = d("main.hmn", 1, 8, 0, "unterminated string literal");
    let out = render(&diag, Some(src));
    assert_eq!(out, "\
error: unterminated string literal
 --> main.hmn:1:8
  |
1 | name = \"hello
  |        ^
");
}

#[test]
fn lex_error_bad_indent() {
    let src = b"CONSTRAINTS safety\n\tNEVER lie\n";
    let diag = d("main.hmn", 2, 1, 0, "tabs not allowed for indentation -- use 2 spaces");
    let out = render(&diag, Some(src));
    assert_eq!(out, "\
error: tabs not allowed for indentation -- use 2 spaces
 --> main.hmn:2:1
  |
2 | \tNEVER lie
  | ^
");
}

#[test]
fn parse_error_duplicate_agent() {
    let src = b"AGENT bot\nAGENT other\n";
    let diag = d("main.hmn", 2, 1, 5, "duplicate AGENT declaration");
    let out = render(&diag, Some(src));
    assert_eq!(out, "\
error: duplicate AGENT declaration
 --> main.hmn:2:1
  |
2 | AGENT other
  | ^~~~~
");
}

#[test]
fn parse_error_expected_indent() {
    let src = b"CONSTRAINTS safety\nFLOW greet\n";
    let diag = d("main.hmn", 1, 1, 11, "expected indented block");
    let out = render(&diag, Some(src));
    assert_eq!(out, "\
error: expected indented block
 --> main.hmn:1:1
  |
1 | CONSTRAINTS safety
  | ^~~~~~~~~~~
");
}

#[test]
fn resolve_error_file_level() {
    let diag = d("main.hmn", 0, 0, 0, "no AGENT declaration found");
    let out = render(&diag, None);
    assert_eq!(out, "\
error: no AGENT declaration found
 --> main.hmn
");
}

#[test]
fn resolve_error_with_location() {
    let src = b"CONSTRAINTS safety\n  NEVER lie\nAGENT bot\n";
    let diag = d("rules.hmn", 3, 1, 5, "AGENT can only appear in main.hmn");
    let out = render(&diag, Some(src));
    assert_eq!(out, "\
error: AGENT can only appear in main.hmn
 --> rules.hmn:3:1
  |
3 | AGENT bot
  | ^~~~~
");
}

#[test]
fn resolve_error_circular_import() {
    let diag = d("a.hmn", 0, 0, 0, "circular import detected: a.hmn -> b.hmn -> a.hmn");
    let out = render(&diag, None);
    assert_eq!(out, "\
error: circular import detected: a.hmn -> b.hmn -> a.hmn
 --> a.hmn
");
}

#[test]
fn compile_error_system_not_found() {
    let diag = d("system.md", 0, 0, 0, "file not found");
    let out = render(&diag, None);
    assert_eq!(out, "\
error: file not found
 --> system.md
");
}

// ── Edge/stress ──

#[test]
fn long_line_no_truncation() {
    let long_line = "x".repeat(200);
    let src = format!("{long_line}\n");
    let diag = d("test.hmn", 1, 150, 0, "far right");
    let out = render(&diag, Some(src.as_bytes()));
    assert!(out.contains(&long_line), "full line should be present");
    let caret_line = format!("{}^", " ".repeat(149));
    assert!(out.contains(&caret_line), "caret at col 150, out: {out}");
}

#[test]
fn error_message_with_quotes() {
    let src = b"bad\n";
    let diag = d("test.hmn", 1, 1, 3, "expected '=' after property name");
    let out = render(&diag, Some(src));
    assert!(out.contains("expected '=' after property name"), "out: {out}");
}

#[test]
fn error_message_with_parens() {
    let diag = d("test.hmn", 0, 0, 0, "import path escapes project root: ./bad (resolved to /etc/bad)");
    let out = render(&diag, None);
    assert!(out.contains("(resolved to /etc/bad)"), "out: {out}");
}

// ── extract_line unit tests ──

#[test]
fn extract_line_basic() {
    let src = b"aaa\nbbb\nccc\n";
    assert_eq!(crate::render::extract_line(src, 1), Some(b"aaa".as_slice()));
    assert_eq!(crate::render::extract_line(src, 2), Some(b"bbb".as_slice()));
    assert_eq!(crate::render::extract_line(src, 3), Some(b"ccc".as_slice()));
    assert_eq!(crate::render::extract_line(src, 4), None);
    assert_eq!(crate::render::extract_line(src, 0), None);
}

#[test]
fn extract_line_no_trailing_newline() {
    let src = b"aaa\nbbb";
    assert_eq!(crate::render::extract_line(src, 1), Some(b"aaa".as_slice()));
    assert_eq!(crate::render::extract_line(src, 2), Some(b"bbb".as_slice()));
    assert_eq!(crate::render::extract_line(src, 3), None);
}

#[test]
fn extract_line_crlf() {
    let src = b"aaa\r\nbbb\r\n";
    assert_eq!(crate::render::extract_line(src, 1), Some(b"aaa".as_slice()));
    assert_eq!(crate::render::extract_line(src, 2), Some(b"bbb".as_slice()));
}

#[test]
fn extract_line_cr_only() {
    let src = b"aaa\rbbb\rccc";
    assert_eq!(crate::render::extract_line(src, 1), Some(b"aaa".as_slice()));
    assert_eq!(crate::render::extract_line(src, 2), Some(b"bbb".as_slice()));
    assert_eq!(crate::render::extract_line(src, 3), Some(b"ccc".as_slice()));
    assert_eq!(crate::render::extract_line(src, 4), None);
}

#[test]
fn extract_line_empty() {
    assert_eq!(crate::render::extract_line(b"", 1), None);
}
