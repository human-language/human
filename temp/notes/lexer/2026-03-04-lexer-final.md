# Lexer Finalization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix five issues identified in the lexer review to produce a final, correct lexer.

**Architecture:** Most changes are in `lexer/src/`. Task 3 also touches `parser/src/parser.rs` because adding a new token kind breaks the parser's IMPORT branch. Changes are otherwise independent and done in order. TDD throughout.

**Tech Stack:** Rust, `cargo test`

---

## Out of Scope (Explicit Decision)

**Issue #4 from the review (byte values vs. Unicode codepoints in ASCII error messages)** is intentionally deferred. The language forbids all non-ASCII characters. The error message quality for an already-rejected file is low priority. If a user hits this they will still get a clear error — the message is just slightly wrong on the codepoint number. This can be revisited when the error-reporting layer is polished.

---

## Known Design Consequence (Not a Bug to Fix Here)

Task 4 adds a test (`flow_body_allcaps_word_is_keyword`) that demonstrates an inherent consequence of Option C: an all-caps word like `NEVER` on a FLOW body line tokenizes as `Keyword::Never`, not an identifier. The lexer is correct — this is exactly what Option C says. However, it means the parser, when processing FLOW body lines, will encounter keyword tokens interspersed with identifiers. The parser must be resilient to this. This is a parser concern, not a lexer bug, and is out of scope for this plan. The test exists to document the behavior, not to flag a defect.

---

## Changes Summary

| # | Issue | Files |
|---|---|---|
| 1 | `str_from` unsoundness | `lexer/src/lexer.rs` |
| 2 | `span.col` is end-of-token | `lexer/src/lexer.rs` |
| 3 | `Package` token kind for IMPORT | `lexer/src/token.rs`, `lexer/src/lexer.rs`, `lexer/src/tests.rs`, `parser/src/parser.rs` |
| 4 | Missing edge case tests | `lexer/src/tests.rs` |
| 5 | Doc contradiction on FLOW body | `temp/docs/guide/basic-tour.md`, `temp/docs/reference/syntax.md` |

---

## Task 1: Fix `str_from` Unsoundness

**Files:**
- Modify: `lexer/src/lexer.rs` (`tokenize` method)

Currently, if `validate_ascii()` finds errors but fewer than `MAX_ERRORS`, tokenizing continues on potentially non-ASCII bytes. The `unsafe str_from` is then called on those bytes — undefined behavior, even though the final result is `Err(errors)`.

**Step 1: Write the failing test**

Add to `lexer/src/tests.rs`:

```rust
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
```

Run: `cargo test --manifest-path lexer/Cargo.toml ascii_error_bails_before_tokenizing`
Expected: PASS (the existing code already returns Err for non-ASCII; this test cements the contract)

**Step 2: Fix `tokenize` in `lexer.rs`**

Change:
```rust
self.validate_ascii();
if self.errors.len() >= MAX_ERRORS {
    return Err(self.errors);
}
```

To:
```rust
self.validate_ascii();
if !self.errors.is_empty() {
    return Err(self.errors);
}
```

This guarantees `str_from` is never called when non-ASCII bytes are present.

**Step 3: Run all tests**

```
cargo test --manifest-path lexer/Cargo.toml
```
Expected: all pass

**Step 4: Commit**

```bash
git add lexer/src/lexer.rs lexer/src/tests.rs
git commit -m "fix: bail on any ASCII error before tokenizing (soundness)"
```

---

## Task 2: Fix `span.col` to Be Start of Token

**Files:**
- Modify: `lexer/src/lexer.rs` (`emit` method)

`emit()` records `self.col` which by call time reflects the end of the token for most token types (keywords, identifiers, text, comments). The fix is to compute `col_at(offset)` inside `emit()` — `offset` is always the token's start byte, so `col_at(offset)` is always correct regardless of where `self.col` happens to be.

**Step 1: Write the failing test**

Add to `lexer/src/tests.rs`:

```rust
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
```

Run: `cargo test --manifest-path lexer/Cargo.toml span_col_is_start_of_token`
Expected: FAIL (currently AGENT reports col 6, support reports col 14 — both are end-of-token)

**Step 2: Fix `emit` in `lexer.rs`**

Change:
```rust
fn emit(&mut self, kind: TokenKind<'a>, offset: usize, len: u16) {
    self.tokens.push(Token {
        kind,
        span: Span {
            offset: offset as u32,
            len,
            line: self.line,
            col: self.col,
        },
    });
}
```

To:
```rust
fn emit(&mut self, kind: TokenKind<'a>, offset: usize, len: u16) {
    let col = self.col_at(offset);
    self.tokens.push(Token {
        kind,
        span: Span {
            offset: offset as u32,
            len,
            line: self.line,
            col,
        },
    });
}
```

`col_at` walks back from `offset` to the preceding `\n` and counts bytes — always gives the start column. This is O(n) per call but acceptable for a config-file lexer.

**Step 3: Run all tests**

```
cargo test --manifest-path lexer/Cargo.toml
```
Expected: all pass including `span_col_is_start_of_token`

**Step 4: Commit**

```bash
git add lexer/src/lexer.rs lexer/src/tests.rs
git commit -m "fix: span.col now records token start column, not end"
```

---

## Task 3: Add `Package` Token Kind for IMPORT Package Names

**Files:**
- Modify: `lexer/src/token.rs`
- Modify: `lexer/src/lexer.rs` (`lex_import_target`)
- Modify: `lexer/src/tests.rs` (2 existing tests updated, 1 new test)
- Modify: `parser/src/parser.rs` line 187 (match `Package` instead of `Ident`)

IMPORT package names like `safety/strict` contain `/` which is not a valid identifier character. Using `Ident` for them is semantically wrong and confuses consumers. A `Package` variant makes the distinction explicit.

**Step 1: Update existing lexer tests to expect `Package`**

In `tests.rs`, update `keyword_import_package` and `keyword_import_package_subpath`:

```rust
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
```

Also add one new test:

```rust
#[test]
fn keyword_import_package_with_dots() {
    assert_eq!(lex("IMPORT org.safety.v2\n"), vec![
        TokenKind::Keyword(Keyword::Import),
        TokenKind::Package("org.safety.v2"),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}
```

Run: `cargo test --manifest-path lexer/Cargo.toml`
Expected: 2 tests FAIL (Package variant doesn't exist yet), 1 test won't compile

**Step 2: Add `Package` to `token.rs`**

In the `TokenKind` enum, add after `Path`:
```rust
Package(&'a str),
```

In `Display` for `TokenKind`, add:
```rust
TokenKind::Package(p) => write!(f, "{}", p),
```

**Step 3: Update `lex_import_target` in `lexer.rs`**

Change the else branch from emitting `Ident` to `Package`:

```rust
} else {
    // Package name: consume until whitespace or EOL.
    // May contain '/', '-', '.' — not a valid identifier, hence Package not Ident.
    while self.pos < self.src.len() && !self.at_eol() && self.src[self.pos] != b' ' {
        self.pos += 1;
    }
    let pkg = self.str_from(start, self.pos).trim_end();
    self.emit(TokenKind::Package(pkg), start, (self.pos - start) as u16);
}
```

**Step 4: Run lexer tests**

```
cargo test --manifest-path lexer/Cargo.toml
```
Expected: all lexer tests pass

**Step 5: Fix the parser**

The parser at `parser/src/parser.rs:187` currently matches `TokenKind::Ident` for import package names. This must be updated to match `TokenKind::Package` or the parser crate will fail to compile (unknown variant) and its tests will fail.

Change:
```rust
TokenKind::Ident(id) => {
    let t = ImportTarget::Package(id.to_string());
    self.advance();
    t
}
```

To:
```rust
TokenKind::Package(pkg) => {
    let t = ImportTarget::Package(pkg.to_string());
    self.advance();
    t
}
```

**Step 6: Run both crates**

```
cargo test --manifest-path lexer/Cargo.toml
cargo test --manifest-path parser/Cargo.toml
```
Expected: all pass in both crates

**Step 7: Commit**

```bash
git add lexer/src/token.rs lexer/src/lexer.rs lexer/src/tests.rs parser/src/parser.rs
git commit -m "feat: add Package token kind for IMPORT package names"
```

---

## Task 4: Add Missing Edge Case Tests

**Files:**
- Modify: `lexer/src/tests.rs`

Five tests for cases documented in `lexer.md` that weren't covered.

**Step 1: Add all five tests**

```rust
// Constraint with only whitespace after keyword — no TEXT token emitted (per lexer.md edge cases)
#[test]
fn modal_constraint_trailing_spaces_only() {
    assert_eq!(lex("  MUST   \n"), vec![
        TokenKind::Indent,
        TokenKind::Keyword(Keyword::Must),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

// FLOW body line with all-caps keyword word tokenizes as Keyword, not Ident.
// This is the correct Option C behavior. NOTE: this means the parser will see
// Keyword::Never inside a FLOW body — the parser must handle this. See "Known
// Design Consequence" at the top of this plan.
#[test]
fn flow_body_allcaps_word_is_keyword() {
    let input = "FLOW x\n  check NEVER something\n";
    let tokens = lex(input);
    assert!(tokens.contains(&TokenKind::Keyword(Keyword::Never)));
    assert!(tokens.contains(&TokenKind::Ident("check")));
    assert!(tokens.contains(&TokenKind::Ident("something")));
}

// Multiple simultaneous DEDENTs: INDENT/DEDENT must stay balanced
#[test]
fn indent_triple_dedent() {
    // 0 -> 2 -> 4 -> 6, then back to 0 — should emit 3 INDENTs and 3 DEDENTs
    let input = "A\n  B\n    C\n      D\nE\n";
    let tokens = lex(input);
    let indent_count = tokens.iter().filter(|t| matches!(t, TokenKind::Indent)).count();
    let dedent_count = tokens.iter().filter(|t| matches!(t, TokenKind::Dedent)).count();
    assert_eq!(indent_count, 3);
    assert_eq!(dedent_count, 3);
    assert_eq!(indent_count, dedent_count);
}

// CRLF in a constraint line: TEXT must not include \r
#[test]
fn modal_text_crlf() {
    assert_eq!(lex("  MUST respond quickly\r\n"), vec![
        TokenKind::Indent,
        TokenKind::Keyword(Keyword::Must),
        TokenKind::Text("respond quickly"),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ]);
}

// SYSTEM with no following token: lexer emits keyword + newline, parser handles the error
#[test]
fn system_no_path() {
    assert_eq!(lex("SYSTEM\n"), vec![
        TokenKind::Keyword(Keyword::System),
        TokenKind::Newline,
        TokenKind::Eof,
    ]);
}
```

**Step 2: Run the new tests**

```
cargo test --manifest-path lexer/Cargo.toml modal_constraint_trailing_spaces_only flow_body_allcaps_word_is_keyword indent_triple_dedent modal_text_crlf system_no_path
```
Expected: all 5 pass. If any fail, the code has a bug — fix the code, not the test.

**Step 3: Run all tests**

```
cargo test --manifest-path lexer/Cargo.toml
```
Expected: all pass

**Step 4: Commit**

```bash
git add lexer/src/tests.rs
git commit -m "test: add edge case tests per lexer spec"
```

---

## Task 5: Fix Doc Contradiction on FLOW Body Tokenization

**Files:**
- Modify: `temp/docs/guide/basic-tour.md` (~line 136)
- Modify: `temp/docs/reference/syntax.md` (~line 83)

The authoritative spec (`lexer.md`) documents Option C: FLOW body lines tokenize as identifiers, NOT as TEXT. These two files incorrectly say FLOW body is captured as TEXT tokens.

**Step 1: Fix `basic-tour.md`**

Find:
```
Each indented line inside a FLOW block is captured as free-form text, not tokenized -- the same modal lexing that applies to constraint prose.
```

Replace with:
```
Each indented line inside a FLOW block tokenizes normally — each word becomes an identifier token. Unlike constraint lines, the lexer does not switch to text-capture mode for FLOW steps (see the Lexer reference for the full rationale).
```

**Step 2: Fix `syntax.md`**

Find:
```
The same capture mode applies to indented lines inside `FLOW` blocks. Each indented line is emitted as a single `TEXT` token representing a pipeline step.
```

Replace with:
```
FLOW body lines are **not** subject to text-capture mode. Each word on an indented FLOW line tokenizes normally as an identifier. See the [Lexer reference](/reference/lexer) for the full rationale (Option C).
```

**Step 3: Confirm no other doc files describe FLOW body as TEXT**

```bash
grep -rn "TEXT" temp/docs/ --include="*.md"
```

Review any hits. Only `lexer.md` should describe TEXT tokens in relation to FLOW — and there it correctly explains that FLOW does NOT use TEXT.

**Step 4: Commit**

```bash
git add temp/docs/guide/basic-tour.md temp/docs/reference/syntax.md
git commit -m "docs: fix FLOW body tokenization description (identifiers, not TEXT)"
```

---

## Final Verification

```
cargo test --manifest-path lexer/Cargo.toml
cargo test --manifest-path parser/Cargo.toml
```

Expected: all tests pass, 0 failed.

The lexer is final when:
- [ ] All existing 49 lexer tests pass
- [ ] 8 new tests pass: 1 soundness + 1 span.col + 1 import-dots + 5 edge cases
- [ ] 2 existing import tests updated to expect `Package`
- [ ] Parser tests still pass after `Package` token kind added
- [ ] `span.col` reflects token start for all token types
- [ ] Non-ASCII bytes always bail before reaching `str_from`
- [ ] IMPORT package names produce `Package` tokens
- [ ] Docs agree with `lexer.md` on FLOW body behavior
- [ ] FLOW-body-keyword design consequence documented in this plan and in the new test
