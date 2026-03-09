## Human Lang — Build Order

1. **Lexer** — tokenize `.hmn` files into tokens (keywords, strings, identifiers, symbols)
2. **Parser** — turn tokens into an AST (abstract syntax tree)
3. **Import Resolver** — follow `@import` references, detect circular deps, build a dependency graph
4. **Compiler/Emitter** — walk the resolved AST and emit a final plain-text prompt or JSON config
5. **CLI** — wire everything together: `human validate`, `human compile`, `human run`, `human init`
6. **Error Formatter** — clean `file.hmn:12:4: error: unexpected token` output with line/col info
7. **Tree-sitter Grammar** — formal grammar file for syntax highlighting in VSCode + Neovim
8. **VSCode Extension** — syntax highlighting, inline error squiggles, autocomplete
9. **npm Package** — publish `human-lang` so people can `npm install -g human-lang`
10. **Brew Formula** — single binary distribution for non-Node users

# pre-release

- update all v0.1.0 -> v1
- code review

# changes
- change the cli tool to use "hmn" not "human"