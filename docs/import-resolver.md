# Import Resolver

## Status

Not yet implemented.
This document is a specification.
It defines the intended behavior of the import resolver before code exists.
Sections marked **[open]** require a design decision before implementation.

## Role

Import resolution for `.hmn` files.
The import resolver is the third stage of the [pipeline](architecture.md), between the [parser](parser.md) and the compiler.

- **Input:** An `HmnFile` (parsed AST of the root file) and access to the filesystem.
- **Output:** A `ResolvedFile` containing the merged result of all transitive imports, or a list of resolution errors.
- **Source:** `import-resolver/src/` (planned)
- **Depends on:** `human-parser`

The resolver performs no tokenization, no parsing, and no code generation.
It reads files, delegates lexing and parsing to the upstream crates, walks the `imports` vector of each `HmnFile`, and produces a single resolved structure.

## Import Syntax

Import declarations are parsed by the [parser](parser.md) into `Import` nodes:

```rust
pub struct Import {
    pub target: ImportTarget,
    pub span: Span,
}

pub enum ImportTarget {
    Path(String),    // ./constraints/safety.hmn, ../shared/common.hmn
    Package(String), // safety, safety/strict, org.safety.v2
}
```

Two forms:

- **Path import:** `IMPORT ./relative/path.hmn` or `IMPORT ../relative/path.hmn`.
  The path is relative to the directory containing the importing file.
- **Package import:** `IMPORT name` or `IMPORT name/subpath`.
  Resolution of package names is an open question (see Packages below).

## Path Resolution

Path imports use filesystem-relative resolution.
No search paths.
No magic.

Given a file at `/project/agents/support.hmn` containing `IMPORT ./constraints/safety.hmn`, the resolver looks for `/project/agents/constraints/safety.hmn`.

Given `IMPORT ../shared/common.hmn` in the same file, the resolver looks for `/project/shared/common.hmn`.

The algorithm:

1. Take the directory of the importing file.
2. Join it with the import path.
3. Canonicalize (resolve `..`, `.`, symlinks).
4. Read the file, lex it, parse it.
5. Recursively resolve its imports.

If the file does not exist, emit an error with the import's `Span` pointing back to the `IMPORT` line in the source.

Path imports must end in `.hmn`.
A path import without the `.hmn` extension is an error. **[open]** -- whether to enforce this or auto-append.

## Root File

The root file is the entry point passed to the CLI (e.g., `human compile main.hmn`).
The resolver does not search for a root file.
It is always explicitly provided.

There is no implicit `main.hmn` convention. **[open]** -- whether `human compile` with no argument should look for `main.hmn` in the current directory.

## Dependency Graph

The resolver builds a directed acyclic graph of file dependencies.

Each node is a parsed `HmnFile` identified by its canonical filesystem path.
Each edge is an `IMPORT` directive.

Files are parsed at most once.
If two files both import the same third file, it is read and parsed once and shared.

## Circular Import Detection

Circular imports are errors.
The resolver detects them during traversal.

Detection: maintain a stack of files currently being resolved (the ancestor chain).
Before resolving a file, check if its canonical path is already on the stack.
If so, emit an error identifying the cycle.

```
main.hmn:3:1: error: circular import detected: main.hmn -> safety.hmn -> main.hmn
```

The error message includes the full cycle path.
Resolution halts for that branch; other branches continue.

## Merge Semantics

After resolving all imports, the resolver merges the dependency graph into a single `ResolvedFile`.

### What Gets Merged

- **CONSTRAINTS blocks:** All constraints from all imported files are collected. Blocks with the same name from different files are merged into a single block. Ordering: imported constraints come before the importing file's constraints (depth-first, left-to-right import order).
- **FLOW blocks:** Collected from all files. Name collisions across files are an error. **[open]** -- whether to allow override/shadowing.
- **TEST blocks:** Collected from all files. No deduplication.
- **AGENT:** Only the root file may declare an `AGENT`. An `AGENT` declaration in an imported file is an error. **[open]** -- whether to relax this for multi-agent projects.
- **SYSTEM:** Attached to the root file's `AGENT`. A `SYSTEM` declaration in an imported file is an error.
- **Properties:** Collected from the root file's `AGENT` only. Imported files do not contribute properties.

### Import Order

Imports are processed in declaration order (top to bottom in the file).
Transitive imports are resolved depth-first.
If file A imports B and C, and B imports D, the resolution order is: D, B, C, A.

This order determines the final ordering of merged constraints and flows.

## ResolvedFile

The output type. **[open]** -- exact structure TBD. Minimum:

```rust
pub struct ResolvedFile {
    pub agent: Option<AgentDecl>,
    pub constraints: Vec<ConstraintsBlock>,
    pub flows: Vec<FlowBlock>,
    pub tests: Vec<TestBlock>,
    pub source_map: HashMap<PathBuf, HmnFile>,
}
```

`source_map` preserves the per-file ASTs for error reporting and debugging.
The merged vectors are the authoritative output passed to the compiler.

## Packages

**[open]** -- Package resolution is not yet specified.

Possible approaches:

1. **Registry:** A central package registry (like npm or crates.io). Heavy. Premature.
2. **Go module-style:** Packages are URLs or paths resolved against a known root. Requires a manifest file.
3. **Local vendor directory:** `IMPORT safety` resolves to `./human_packages/safety/main.hmn` or similar. Simple. No network. Explicit.
4. **No packages in v0.1:** Only path imports are supported. Package imports are a parse-level concept but resolution is deferred to a later version.

Option 4 is the most conservative.
The lexer and parser already distinguish `Path` from `Package` tokens, so the syntax is forward-compatible.
The resolver can reject `Package` imports with a clear error until a package system is designed.

## Errors

### Error Type

```rust
pub struct ResolveError {
    pub span: Span,
    pub file: PathBuf,
    pub message: String,
}
```

The `span` points to the `IMPORT` declaration that caused the error.
The `file` is the path of the file containing the import.

`display` format:

```
agents/support.hmn:3:1: error: file not found: ./constraints/missing.hmn
```

### Error Categories

- **File not found:** Import target does not exist on the filesystem.
- **Circular import:** Import creates a cycle in the dependency graph.
- **Duplicate AGENT:** An imported file contains an `AGENT` declaration.
- **Duplicate SYSTEM:** An imported file contains a `SYSTEM` declaration.
- **Duplicate FLOW name:** Two files define a `FLOW` block with the same name. **[open]**
- **Package not supported:** A `Package` import is used but package resolution is not yet implemented.
- **Read error:** Filesystem I/O failure (permissions, encoding, etc.).
- **Lex/parse error in imported file:** The imported file has syntax errors. The resolver reports the original lex/parse errors with the imported file's path.

## Invariants

These must hold when the resolver is implemented:

- Each file is read and parsed at most once per resolution.
- Circular imports are always detected and rejected.
- The resolver never modifies an `HmnFile`. It reads them and produces a new `ResolvedFile`.
- Only the root file may contain `AGENT` and `SYSTEM` declarations.
- Import order is deterministic: same input always produces same output.
- All errors carry the file path and span of the originating `IMPORT` line.

## Design Principles

These are inherited from the [Dennis mentorship prompt](../temp/prompts/agents/dennis/dennis.md) Phase 4:

- Mirror how C headers work: simple, explicit, no magic.
- If the import system requires a runtime to resolve, the design has failed.
- Imports are always explicit. No implicit imports.
- `.hmn` files are plain text, composable, and pipeable.

## Open Questions Summary

1. **Package resolution strategy.** No design exists. Path imports only for v0.1?
2. **Root file convention.** Should `human compile` with no argument default to `main.hmn`?
3. **`.hmn` extension enforcement.** Should path imports require the `.hmn` extension or auto-append it?
4. **FLOW name collisions.** Error, override, or merge?
5. **Multi-agent projects.** Should imported files be allowed to declare `AGENT`?
6. **`ResolvedFile` exact structure.** The sketch above is a starting point.
7. **Error reporting for transitive imports.** How to display the import chain in error messages.
