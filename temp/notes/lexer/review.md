# Lexer Review Notes

Source of truth: `temp/docs/reference/lexer.md` and `temp/docs/reference/syntax.md`.

---

## Overall Assessment

The lexer is solid. Core logic is correct: all 16 keywords map, modal capture works, INDENT/DEDENT stack is correct, comment detection is correct, CRLF is handled, ASCII validation works, string escapes work. The test coverage is good for the happy path.

Issues below are ordered by severity.

---

## Issues

### 1. Doc Contradiction on FLOW Body Tokenization (Docs bug, not code bug)

`lexer.md` documents the authoritative design decision as **Option C**: FLOW body lines tokenize normally as identifiers. The implementation correctly follows this.

But `basic-tour.md` (line 136) says:
> Each indented line inside a FLOW block is captured as free-form text, not tokenized — the same modal lexing that applies to constraint prose.

And `syntax.md` (line 83) says:
> The same capture mode applies to indented lines inside FLOW blocks. Each indented line is emitted as a single TEXT token representing a pipeline step.

These two guide files directly contradict `lexer.md` and the implementation. The test `flow_body_tokenizes_normally` confirms the implementation is right. `basic-tour.md` and `syntax.md` need to be corrected to say FLOW body lines tokenize as identifiers (Option C).

**Action**: Fix the two doc files, not the code.

---

### 2. `span.col` Is the End of the Token, Not the Start

In `emit()`, `self.col` is used for `span.col`. But `self.col` is updated *as scanning progresses* — by the time `emit()` is called, it usually reflects the position *after* the token, not the start.

Examples:
- Keywords/identifiers: `consume_ident()` calls `self.col = self.col_at(self.pos)` (post-ident pos), then `emit()` is called — `span.col` = end column.
- Text tokens: `skip_to_eol()` sets `self.col = col_at(self.pos)` (EOL), then `emit()` — `span.col` = EOL.
- Comments: same pattern.
- Numbers: `emit()` is called *before* `self.col = self.col_at(self.pos)`, so `span.col` happens to be the start column (correct for numbers — a lucky accident of ordering).

The `span.offset` field is always correct and can be used to recompute position. But `span.col` is unreliable for anything that uses it for error messages. Error messages that read `span.col` will show the wrong column for keywords, identifiers, and text tokens.

**Fix**: Save the col at the *start* of each token, before advancing `self.pos`, and pass it explicitly to `emit()`. Or compute `col_at(offset)` at emit time.

---

### 3. `str_from` Is Unsound When ASCII Errors Exist But < MAX_ERRORS

```rust
fn str_from(&self, start: usize, end: usize) -> &'a str {
    // Safe: we validated ASCII in the pre-pass
    unsafe { std::str::from_utf8_unchecked(&self.src[start..end]) }
}
```

The comment says "safe because of ASCII validation", but if `validate_ascii()` found errors (just fewer than `MAX_ERRORS`), tokenizing continues on bytes that may be non-ASCII. `str_from` is called on those bytes, producing invalid `&str` slices — undefined behavior in Rust.

In practice the final result is `Err(errors)` so nothing uses the invalid strings. But undefined behavior is still triggered.

**Fix**: Either bail out on any ASCII error (change the `>= MAX_ERRORS` guard to `> 0`), or use `std::str::from_utf8()` with a fallback. The first option is simpler and appropriate for a config language where non-ASCII is never valid.

---

### 4. `validate_ascii` Reports Byte Values, Not Unicode Codepoints

```rust
format!("invalid character U+{:04X} -- .hmn files must be ASCII only", b as u32)
```

For a UTF-8 sequence like the emoji `🖐` (bytes `0xF0 0x9F 0x91 0x96`), this reports four separate errors: `U+00F0`, `U+009F`, `U+0091`, `U+0096`. The spec example shows `U+1F44B` (the actual Unicode codepoint). Minor UX issue but the error messages look confusing.

**Fix**: Detect multi-byte UTF-8 sequences in `validate_ascii`, decode the codepoint, and report once per character. Or just say "invalid byte 0xF0" instead of pretending it's a Unicode codepoint.

---

### 5. IMPORT Package Names Emitted as `Ident` Despite Invalid Identifier Content

`IMPORT safety/strict` emits `Ident("safety/strict")`. The `Ident` token kind is defined to match `[a-zA-Z_][a-zA-Z0-9_]*` (from the syntax reference), but package names can contain `/`, `-`, `.`. Consumers of the token stream need to know that `Ident` after `IMPORT` is a "loose identifier" that may not follow identifier grammar.

**Fix**: Either add a `Package` token kind, or document explicitly (in code comments and docs) that `Ident` after `IMPORT` is a package name with relaxed rules.

---

### 6. Missing Tests

These spec-defined edge cases don't have explicit tests:

| Case | Spec location | What to test |
|---|---|---|
| `MUST   ` (trailing spaces, empty after trim) | `lexer.md` edge cases | Should emit `MUST`, `NEWLINE` — no TEXT |
| FLOW body line with all-caps keyword | `lexer.md` Option C | e.g. `FLOW x\n  NEVER do this\n` — `NEVER` tokenizes as `Keyword::Never`, not `Ident` |
| Multiple DEDENTs at once (3+ levels) | `syntax.md` indentation | Stack should emit N DEDENTs correctly |
| `SYSTEM` with no path | `keywords.md` | Should emit `KEYWORD(System)` then `NEWLINE` only |
| Tab in middle of line (not indentation) | `syntax.md` | Tab is legal ASCII but note: would hit `unexpected character` error |
| CRLF in constraint line | `lexer.md` | `MUST respond\r\n` — TEXT should not contain `\r` |

The `modal_empty_no_text` test covers `NEVER\n` with no trailing content, but not `NEVER   \n` with whitespace only. The code handles it correctly (skip_spaces + at_eol check), it's just untested.

---

## Confirmed Correct

- All 16 keywords in `keyword_from_str` match the spec exactly
- `is_constraint_keyword` covers the right 5 keywords
- Modal capture: emits TEXT for rest-of-line, trims leading/trailing whitespace, no TEXT emitted when empty
- Text with `#`, `$`, `>`, `%`, numbers inside constraint prose — all captured verbatim (no tokenization)
- Keyword inside text (`NEVER MUST do X`) → TEXT captures everything including the keyword
- INDENT/DEDENT stack: blank lines don't trigger, balance maintained at EOF
- Tab in indentation: error emitted, recovery continues
- Odd indentation (not multiple of 2): error emitted
- INDENT doesn't match any outer level: error emitted
- Comments: `#` as first non-whitespace → COMMENT token (body without `#` prefix)
- `#` inside constraint text → captured as prose (correct per spec)
- String escapes: `\"` and `\\` handled, other `\x` passed through as-is
- Unterminated string → error
- Negative numbers, floats, integers
- `true` / `false` as booleans (not keywords, not identifiers)
- Paths: `./` and `../` prefix, to EOL
- CRLF: handled in both `validate_ascii`, `skip_to_eol`, `skip_newline`, `finish_line`
- ASCII validation: rejects non-printable, non-tab bytes
- EOF without trailing newline: no crash, no NEWLINE emitted, EOF emitted
- Empty file: just EOF
- MAX_ERRORS guard prevents runaway error accumulation
