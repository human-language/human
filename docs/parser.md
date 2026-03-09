# Parser

## Role

Syntactic analysis for `.hmn` files.
The parser is the second stage of the [pipeline](architecture.md).

- **Input:** `&[Token<'a>]` -- a token slice produced by the [lexer](lexer.md).
- **Output:** `Result<HmnFile, Vec<ParseError>>`
- **Source:** `parser/src/`
- **Depends on:** `human-lexer` (path dependency `../lexer`)

The parser performs no tokenization, no file I/O, and no import resolution.
It converts a flat token stream into a tree of AST nodes.
Everything downstream consumes the AST.

## Public API

Defined in `parser/src/lib.rs`.

```rust
pub fn parse<'a>(tokens: &'a [Token<'a>]) -> Result<HmnFile, Vec<ParseError>>
```

One entry point.
Returns the complete file AST on success.
Returns a collected error vector on failure.
The parser always attempts to parse the entire file; it does not bail on the first error.

## AST

Defined in `parser/src/types.rs`.
All AST nodes own their strings (`String`, not `&str`).
No lifetime parameter on any AST type.
Every node that maps to a source location carries a `span: Span` field (re-exported from `human-lexer`).

### HmnFile

The root node.

```rust
pub struct HmnFile {
    pub imports: Vec<Import>,
    pub agent: Option<AgentDecl>,
    pub constraints: Vec<ConstraintsBlock>,
    pub flows: Vec<FlowBlock>,
    pub tests: Vec<TestBlock>,
}
```

All fields default to empty/`None`.
An empty file produces an empty `HmnFile`.

### Import

```rust
pub struct Import {
    pub target: ImportTarget,
    pub span: Span,
}

pub enum ImportTarget {
    Path(String),
    Package(String),
}
```

`Path` corresponds to `TokenKind::Path` (starts with `./` or `../`).
`Package` corresponds to `TokenKind::Package` (bare name, may contain `/`, `-`, `.`).

### AgentDecl

```rust
pub struct AgentDecl {
    pub name: String,
    pub properties: Vec<Property>,
    pub system: Option<SystemDecl>,
    pub span: Span,
}
```

At most one `AgentDecl` per file.
The parser rejects duplicates.
The name is an identifier token.
The body is optional -- `AGENT minimal\n` with no indented block is valid.

### SystemDecl

```rust
pub struct SystemDecl {
    pub path: String,
    pub span: Span,
}
```

Attached to the `AgentDecl`.
Accepted at two positions: top-level (after `AGENT`) and indented inside the `AGENT` body.
At most one `SYSTEM` per agent.
`SYSTEM` without a preceding `AGENT` is an error.

### Property

```rust
pub struct Property {
    pub key: String,
    pub value: Value,
    pub span: Span,
}

pub enum Value {
    Str(String),
    Number(f64),
    Bool(bool),
    Path(String),
}
```

Properties appear inside an `AGENT` body.
Syntax: `key = value`.
The key is an identifier.
The value is a string literal, number, boolean, or path.

### ConstraintsBlock

```rust
pub struct ConstraintsBlock {
    pub name: String,
    pub constraints: Vec<Constraint>,
    pub span: Span,
}
```

Multiple `CONSTRAINTS` blocks per file are allowed.
Each has a name (identifier) and a required indented body.

### Constraint

```rust
pub struct Constraint {
    pub level: ConstraintLevel,
    pub text: String,
    pub span: Span,
}

pub enum ConstraintLevel {
    Never,
    Must,
    Should,
    Avoid,
    May,
}
```

Each constraint is a level keyword followed by a `Text` token.
The text is the prose captured by the lexer's modal text mode.
A constraint keyword without text is a parse error.

### FlowBlock

```rust
pub struct FlowBlock {
    pub name: String,
    pub steps: Vec<FlowStep>,
    pub span: Span,
}

pub struct FlowStep {
    pub text: String,
    pub span: Span,
}
```

Multiple `FLOW` blocks per file are allowed.
Each has a name (identifier) and a required indented body.
Steps are one per line.

### FLOW Step Reassembly

FLOW body lines tokenize normally (see [lexer.md](lexer.md), Modal Lexing).
Each word becomes a separate token -- identifiers, keywords, numbers, booleans, or other token kinds.
The parser collects all tokens on the line and joins their display representations with spaces.
The result is stored as `FlowStep.text`.

This means `check NEVER conditions` produces `text: "check NEVER conditions"` even though `NEVER` arrives as `Keyword(Never)`, not `Ident`.

### TestBlock

```rust
pub struct TestBlock {
    pub inputs: Vec<TestInput>,
    pub expects: Vec<TestExpect>,
    pub span: Span,
}
```

`TEST` takes no name.
Multiple `TEST` blocks per file are allowed.
The body contains `INPUT` and `EXPECT` lines in any order.
A `TEST` block with no `INPUT` or no `EXPECT` is valid at the parse level.

### TestInput

```rust
pub struct TestInput {
    pub value: String,
    pub span: Span,
}
```

`INPUT` followed by a quoted string.

### TestExpect

```rust
pub struct TestExpect {
    pub negated: bool,
    pub op: TestOp,
    pub value: String,
    pub span: Span,
}

pub enum TestOp {
    Contains,
    Matches,
}
```

Forms:
- `EXPECT CONTAINS "string"`
- `EXPECT NOT CONTAINS "string"`
- `EXPECT MATCHES "pattern"`
- `EXPECT NOT MATCHES "pattern"`

`NOT` is optional.
The operator is `CONTAINS` or `MATCHES`.
The value is always a quoted string.
Any other `EXPECT` form is a parse error.

## Parse Rules

### Top-Level Dispatch

`parse_file` loops over tokens, skipping `Newline` and `Comment` tokens between declarations.
It dispatches on the top-level keyword:

- `IMPORT` -> `parse_import`
- `AGENT` -> `parse_agent`
- `SYSTEM` -> attaches to existing agent
- `CONSTRAINTS` -> `parse_constraints`
- `FLOW` -> `parse_flow`
- `TEST` -> `parse_test`

Any other token at top level is an error.
The parser skips to the next top-level keyword and continues.

Top-level keywords: `AGENT`, `CONSTRAINTS`, `TEST`, `FLOW`, `SYSTEM`, `IMPORT`.

### Block Structure

Blocks (`CONSTRAINTS`, `FLOW`, `TEST`, `AGENT` body) use `Indent`/`Dedent` tokens for scoping.
The parser expects `Indent` after the block header line.
If `Indent` is missing, an error is emitted and the block is returned empty.
Inside a block, the parser loops until `Dedent` or `Eof`.

`Newline` and `Comment` tokens are skipped between block-body lines.
Comments are not represented in the AST.

### AGENT Body

The `AGENT` body is optional.
If `Indent` follows the header, the parser enters the body and accepts:
- `SYSTEM` declarations
- Property assignments (`ident = value`)

Any other token in the body is an error; the parser skips to the next newline and continues.

### SYSTEM Positioning

`SYSTEM` is accepted at two positions:

1. **Top-level**, immediately after `AGENT` (same indentation level). `parse_file` handles this by mutating the already-parsed `AgentDecl`.
2. **Inside AGENT body** (indented). `parse_agent` handles this.

Both positions attach the `SystemDecl` to `AgentDecl.system`.
Duplicates at either position are rejected.
`SYSTEM` before any `AGENT` is an error.

### Declaration Order

The parser does not enforce ordering between top-level declarations.
`IMPORT` after `AGENT`, `CONSTRAINTS` before `AGENT`, etc. are all accepted.
The `HmnFile` struct collects each declaration type into its own vector (or `Option` for `agent`).

## Error Recovery

The parser does not stop on the first error.
It attempts to recover and continue parsing to report multiple errors in one pass.

### Recovery Strategies

- **skip_to_newline:** Advance past tokens until `Newline`, `Dedent`, or `Eof`. Used after errors inside block bodies.
- **skip_to_toplevel:** Advance past tokens until a top-level keyword or `Eof`. Used after errors at the file level.

When a block parser (`parse_import`, `parse_agent`, etc.) encounters an error, it records the error, skips to a recovery point, and returns `None`.
The top-level loop continues with the next declaration.

When a body-line parser (`parse_constraint`, `parse_flow_step`, etc.) encounters an error, it records the error, skips to the next line, and continues the body loop.

### Error Cap

`MAX_ERRORS = 10`.
When 10 errors accumulate, the parser stops all loops and returns `Err(errors)`.
This prevents runaway error cascades from a single structural mistake.

## Errors

### ParseError

Defined in `parser/src/error.rs`.

```rust
pub struct ParseError {
    pub span: Span,
    pub message: String,
}
```

`display_with_file(filename)` produces:

```
file.hmn:12:4: error: message text here
```

The `span` is the span of the token that triggered the error (via `self.span()` at the current parser position).

### Error Messages

- `duplicate AGENT declaration`
- `SYSTEM without preceding AGENT declaration`
- `duplicate SYSTEM declaration`
- `expected identifier after AGENT`
- `expected identifier after CONSTRAINTS`
- `expected identifier after FLOW`
- `expected file path or package name after IMPORT`
- `expected file path after SYSTEM`
- `expected indented block`
- `expected newline`
- `expected '=' after property name`
- `expected value after '='`
- `expected property name`
- `expected constraint keyword (NEVER/MUST/SHOULD/AVOID/MAY), got: ...`
- `expected constraint level keyword`
- `expected constraint text after level keyword`
- `expected flow step text`
- `expected INPUT or EXPECT in TEST block, got: ...`
- `expected quoted string after INPUT`
- `unsupported EXPECT form`
- `expected quoted string after CONTAINS/MATCHES`
- `unexpected token: ...`
- `unexpected token in AGENT body: ...`

## Parser Internals

### Cursor

The parser maintains a position (`pos: usize`) into the token slice.
Three cursor methods:

- `peek()` -- returns `&TokenKind` at current position, or `Eof` past the end.
- `span()` -- returns `Span` at current position.
- `advance()` -- returns current `&Token` and increments position. Clamps at end of slice.

### Token Skipping

- `skip_newlines()` -- advances past `Newline` and `Comment` tokens. Called between declarations and between block-body lines.
- `expect_newline()` -- consumes one `Newline`, accepts `Eof`, errors on anything else.
- `expect_indent()` -- consumes one `Indent` or emits "expected indented block" error.

## Invariants

- `HmnFile` always contains at most one `AgentDecl`.
- `SYSTEM` is always attached to `AgentDecl`, never free-standing.
- Every AST node that maps to source carries a `Span`.
- All strings in the AST are owned (`String`). No lifetime parameter leaks into the AST.
- Comments are consumed and discarded. They do not appear in the AST.
- The parser never panics on valid or invalid input. All error paths record a `ParseError` and recover.
- On success (`Ok(HmnFile)`), the error vector is empty.
- On failure (`Err(Vec<ParseError>)`), the error vector is non-empty and contains at most 10 errors.

## Open Questions

- **TEST block without INPUT or EXPECT is valid at parse level.**
  Whether this should be a warning or error is a semantic question for the compiler or test runner.
  The parser does not enforce it.
- **Declaration ordering is not enforced.**
  Whether `IMPORT` should be required before other declarations is a style question.
  The parser currently accepts any order.
- **AGENT body properties have no schema validation.**
  `model = "gpt-4"` and `foo = "bar"` are both accepted.
  Key validation is a downstream concern.
