# Documentation Rules

This directory contains internal engineering documentation for the Human compiler and runtime.
These are contributor-facing reference documents -- not user guides, not marketing, not tutorials.
User-facing documentation lives in `temp/docs/`.

## Tone

Bell Labs internal memo.
Terse, exact, mechanical.
No enthusiasm.
No hedging.
No filler.
No startup language.
Write for someone who will read the source next.

## Format

Markdown.
One `# Title` per file.
`##` for major sections.
`###` for subsections.
No deeper nesting.

## Headings

Noun phrases or short declaratives.
Not questions.
Not actions.
"Token Model", not "How Tokens Work" or "Understanding the Token Model".

## Code Examples

Use fenced code blocks with language tag (`rust`, `human`, or plain).
Show real types and real values from the implementation.
No pseudocode unless the implementation does not yet exist.
Reference source files (e.g., `lexer/src/token.rs`) when citing implementation detail.

## Diagrams

Only when data flow or sequencing is clearer as a diagram than as prose.
Use mermaid or ASCII art.
No images.

## Invariants

State as declarative sentences.
"The token stream always ends with Eof."
Not "Make sure the token stream ends with Eof."

## Specification vs. Implementation

Specification says what must be true.
Implementation says how it is achieved.
Prefer specification.
When implementation detail is necessary, name the source file so the reader knows where to look.

## Tense

Present tense.
"The lexer emits INDENT tokens."
Not "The lexer will emit" or "The lexer should emit."

## Line Length

No hard wrap.
One sentence per line in source (for clean diffs).

## Links

Relative links between docs files: `[architecture](architecture.md)`.
No absolute URLs to the repo.

## Updates

When implementation changes, the corresponding doc must be updated in the same commit.

## Quality Bar

A doc is done when:

- Every statement is true of the current implementation (or explicitly marked as planned/unbuilt).
- No sentence can be removed without losing information.
- Types, variant names, field names, and function names match the source exactly.
- A contributor can find the answer to "where does X happen" within 30 seconds.
- When the implementation changes, exactly one section of exactly one doc needs updating.
- Each doc can be read without having read the others, though it may link to them for detail.
- Unbuilt subsystems are marked as unbuilt. Open questions are listed. Nothing is implied to exist that does not.
