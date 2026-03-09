# Lexer

## Role

Lexical analysis for `.hmn` files.
The lexer is the first stage of the [pipeline](architecture.md).

- **Input:** `&[u8]` -- raw bytes of a single `.hmn` file.
- **Output:** `Result<Vec<Token<'a>>, Vec<LexError>>`
- **Source:** `lexer/src/`

The lexer performs no file I/O, no path resolution, and no AST construction.
It converts bytes to tokens.
Everything downstream consumes the token stream.

## Token Model

Defined in `lexer/src/token.rs`.

```rust
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub span: Span,
}
```

### TokenKind

14 variants:

- `Keyword(Keyword)` -- one of 16 reserved words (see Keywords below).
- `Ident(&'a str)` -- identifier matching `[a-zA-Z_][a-zA-Z0-9_]*`, not a keyword, not a boolean.
- `Text(&'a str)` -- free-form prose captured after a constraint keyword. Borrows from source.
- `Str(String)` -- double-quoted string literal. Owned because escape processing may modify content.
- `Number(f64)` -- integer or float literal.
- `Bool(bool)` -- `true` or `false`.
- `Path(&'a str)` -- file path starting with `./` or `../`.
- `Package(&'a str)` -- import package name. May contain `/`, `-`, `.` (not valid identifier characters).
- `Equals` -- the `=` character.
- `Comment(&'a str)` -- comment body (without `#` prefix, trimmed).
- `Indent` -- indentation level increased.
- `Dedent` -- indentation level decreased.
- `Newline` -- end of a non-blank line.
- `Eof` -- end of input.

## Span

Every token carries a `Span` recording its position in the source.

```rust
pub struct Span {
    pub offset: u32,  // byte offset from start of input
    pub len: u16,     // byte length of token
    pub line: u32,    // 1-indexed line number
    pub col: u16,     // 1-indexed column, start of token
}
```

`col` is computed by `col_at(offset)` at emit time.
It walks backwards from the token's start byte to the preceding `\n` and counts the distance.
This is O(n) per call but acceptable for a configuration-file lexer.

Synthetic tokens (`Indent`, `Dedent`, `Newline`, `Eof`) have `len: 0`.
Their `offset` points to the position where they logically occur.

## Keywords

16 keywords, all uppercase.
Matched by `keyword_from_str` in `lexer/src/token.rs`.

### Structure Keywords

`AGENT`, `CONSTRAINTS`, `TEST`, `FLOW` -- open blocks.

### Module Keywords

`SYSTEM`, `IMPORT` -- single-line declarations referencing external files or packages.

### Constraint Keywords

`NEVER`, `MUST`, `SHOULD`, `AVOID`, `MAY` -- trigger modal text capture.
Identified by `is_constraint_keyword`.

### Test Keywords

`INPUT`, `EXPECT`, `NOT`, `CONTAINS`, `MATCHES` -- used inside `TEST` blocks.

If an identifier matches a keyword, the lexer emits `Keyword`, not `Ident`.
There is no context-sensitivity -- `NEVER` is always a keyword token, even on a FLOW body line.

## Lexer Phases

The lexer runs in two phases.
Both are in `lexer/src/lexer.rs`.

### Phase 1: ASCII Validation

`validate_ascii` scans the entire input before any tokenization.
It rejects any byte outside: printable ASCII (`0x20`-`0x7E`), tab (`0x09`), newline (`0x0A`), carriage return (`0x0D`).

If any invalid byte is found, the lexer returns `Err(errors)` immediately.
No tokenization occurs.

This guarantees the safety invariant: `str_from` uses `std::str::from_utf8_unchecked` and is only called on bytes that have passed ASCII validation.

### Phase 2: Line-by-Line Tokenization

`lex_line` processes one line at a time:

1. Count leading spaces. Reject tabs in indentation.
2. If the line is blank or at EOL, skip it.
3. Validate indentation is a multiple of 2 spaces.
4. Emit `Indent`/`Dedent` tokens via `update_indent`.
5. If `#` is the first non-whitespace character, emit a `Comment` token.
6. Otherwise, call `lex_line_content` for normal tokenization.
7. Emit `Newline` and advance past the line terminator.

At end of input, `drain_indents` emits `Dedent` for each remaining level on the indent stack, then `Eof` is emitted.

## Indentation

The lexer maintains `indent_stack: Vec<u16>`, initialized to `[0]`.

- When a line's leading spaces exceed the top of the stack, `Indent` is emitted and the new level is pushed.
- When a line's leading spaces are less than the top, `Dedent` tokens are emitted (one per popped level) until the stack matches.
- If the dedented level does not match any level on the stack, an error is emitted.
- Odd indentation (not a multiple of 2) is an error but the lexer continues.
- Tabs in indentation are an error. The tab is counted as 1 space for recovery.
- Blank lines do not affect indentation.
- At EOF, `drain_indents` pops all remaining levels and emits one `Dedent` per level.

## Modal Lexing

The lexer has exactly one modal rule.

**Trigger:** After emitting a constraint keyword (`NEVER`, `MUST`, `SHOULD`, `AVOID`, `MAY`), the lexer switches to text-capture mode for the remainder of that line.

**Behavior:** Everything from the keyword to the end of the line (trimmed of leading and trailing whitespace) is emitted as a single `Text` token.
No further tokenization occurs on that line.
`30` is not a number literal, `#SUP` is not a comment -- they are prose.

**Empty text:** If the rest of the line is empty or whitespace-only after trimming, no `Text` token is emitted.
The parser treats this as an error (constraint keyword without constraint text).

**Everything else tokenizes normally.**
This is the Option C design decision documented in `temp/docs/reference/lexer.md`.

### Design Consequence

FLOW body lines tokenize normally.
A word like `NEVER` on a FLOW body line produces `Keyword(Never)`, not `Ident("NEVER")`.
The parser handles this by collecting all token kinds on a FLOW step line and joining their text representations.
This is a known consequence of Option C, not a bug.

## IMPORT Targets

`lex_import_target` in `lexer/src/lexer.rs` runs after emitting `Keyword(Import)`.

- If the target starts with `./` or `../`, it is a file path.
  Consumed to end of line, emitted as `Path`.
- Otherwise, it is a package name.
  Consumed to the next space or end of line, emitted as `Package`.
  Package names may contain `/`, `-`, `.` -- characters not legal in identifiers.

## SYSTEM Target

`lex_system_target` runs after emitting `Keyword(System)`.
The target is always a file path.
Consumed to end of line, emitted as `Path`.

## Comments

A `#` character is a comment only when it is the first non-whitespace character on a line (after indentation).

The lexer emits `Comment(body)` where `body` is the text after `#` with the leading space (if present) and trailing whitespace stripped.

`#` appearing inside constraint text is prose, not a comment.
The comment check happens before keyword detection in `lex_line`, so a line starting with `#` is always a comment regardless of what follows.

## String Literals

Double-quote delimited.
Lexed by `lex_string` in `lexer/src/lexer.rs`.

Escape sequences:
- `\"` -- literal double quote.
- `\\` -- literal backslash.
- Any other `\x` sequence is passed through as-is (both characters preserved).

Unterminated strings (no closing `"` before end of line or end of input) produce an error.

The result is `Str(String)` -- owned, because escape processing may differ from source bytes.

## Number Literals

Lexed by `lex_number`.

Formats: integer (`[0-9]+`), float (`[0-9]+.[0-9]+`), negative (`-[0-9]+`, `-[0-9]+.[0-9]+`).
Negative numbers require `-` immediately followed by a digit.
Stored as `Number(f64)`.

No hex, no scientific notation, no underscores, no leading dot.

## Boolean Literals

`true` and `false` (lowercase only).
Detected during identifier lexing.
Emitted as `Bool(true)` or `Bool(false)`.
They are not keywords and not identifiers.

## Path Literals

Starts with `./` or `../`.
Consumed to end of line or next space.
Emitted as `Path(&'a str)`, borrowing from source.

Path tokens appear after `SYSTEM`, `IMPORT`, and in property values.
`lex_path` handles paths encountered in normal token position.
`lex_import_target` and `lex_system_target` handle paths in those specific contexts.

## Errors

### LexError

Defined in `lexer/src/error.rs`.

```rust
pub struct LexError {
    pub line: u32,
    pub col: u16,
    pub message: String,
}
```

`display_with_file(filename)` produces:

```
file.hmn:12:4: error: message text here
```

### Error Cap

`MAX_ERRORS = 10`.
When 10 errors accumulate, the lexer stops processing and returns `Err(errors)`.

### Error Categories

- **ASCII validation:** `invalid character U+00xx -- .hmn files must be ASCII only`
- **Indentation (tabs):** `tabs not allowed for indentation -- use 2 spaces`
- **Indentation (odd):** `indentation must be a multiple of 2 spaces`
- **Indentation (mismatch):** `indentation does not match any outer level`
- **Unterminated string:** `unterminated string literal`
- **Invalid number:** `invalid number literal 'xxx'`
- **Unexpected character:** `unexpected character 'x'`

## Invariants

- The token stream always ends with `Eof`.
- `Indent` and `Dedent` tokens are always balanced. Every `Indent` has a matching `Dedent`, including those emitted by `drain_indents` at EOF.
- `str_from` is never called on non-ASCII input. ASCII validation either passes cleanly or the lexer returns before reaching any tokenization code.
- `Text` tokens only appear immediately after a constraint keyword (`NEVER`/`MUST`/`SHOULD`/`AVOID`/`MAY`).
- `Package` tokens only appear immediately after `Keyword(Import)`.
- `Newline` is emitted for every non-blank line that ends with `\n` or `\r\n`.
- No `Newline` is emitted for EOF without a trailing newline.
- CRLF (`\r\n`) is treated as a single line terminator. `\r` alone also terminates a line. Neither `\r` nor `\r\n` appear in token text -- `Text` and `Comment` content is trimmed before the carriage return.

## Open Questions

- **ASCII error messages report byte values, not Unicode codepoints.**
  A UTF-8 multi-byte sequence like `🖐` (bytes `0xF0 0x9F 0x91 0x96`) produces four separate errors (`U+00F0`, `U+009F`, etc.) instead of one error with the actual codepoint `U+1F44B`.
  Low priority -- the file is already rejected.
  Deferred.
- **FLOW body keywords tokenize as `Keyword`, not `Ident`.**
  This is correct per the Option C design.
  Whether this should change long-term depends on whether FLOW body parsing requirements evolve.
  Currently the parser handles it.
  No action needed unless FLOW semantics change.
