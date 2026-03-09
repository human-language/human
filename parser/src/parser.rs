use human_lexer::{Token, TokenKind, Keyword, Span, is_constraint_keyword};
use crate::error::ParseError;
use crate::types::*;

const MAX_ERRORS: usize = 10;

struct Parser<'a> {
    tokens: &'a [Token<'a>],
    pos: usize,
    errors: Vec<ParseError>,
}

pub fn parse<'a>(tokens: &'a [Token<'a>]) -> Result<HmnFile, Vec<ParseError>> {
    let mut p = Parser { tokens, pos: 0, errors: Vec::new() };
    let file = p.parse_file();
    if p.errors.is_empty() {
        Ok(file)
    } else {
        Err(p.errors)
    }
}

// --- Cursor helpers ---

impl<'a> Parser<'a> {
    fn peek(&self) -> &TokenKind<'a> {
        self.tokens.get(self.pos).map(|t| &t.kind).unwrap_or(&TokenKind::Eof)
    }

    fn span(&self) -> Span {
        self.tokens.get(self.pos).map(|t| t.span).unwrap_or(Span { offset: 0, len: 0, line: 0, col: 0 })
    }

    fn advance(&mut self) -> &Token<'a> {
        let tok = &self.tokens[self.pos.min(self.tokens.len() - 1)];
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek(), TokenKind::Eof)
    }

    fn skip_newlines(&mut self) {
        loop {
            match self.peek() {
                TokenKind::Newline | TokenKind::Comment(_) => { self.advance(); }
                _ => break,
            }
        }
    }

    fn expect_newline(&mut self) {
        match self.peek() {
            TokenKind::Newline => { self.advance(); }
            TokenKind::Eof => {}
            _ => { self.error("expected newline"); }
        }
    }

    fn expect_indent(&mut self) -> bool {
        if matches!(self.peek(), TokenKind::Indent) {
            self.advance();
            true
        } else {
            self.error("expected indented block");
            false
        }
    }

    fn error(&mut self, message: impl Into<String>) {
        if self.errors.len() < MAX_ERRORS {
            self.errors.push(ParseError::new(self.span(), message));
        }
    }

    fn too_many_errors(&self) -> bool {
        self.errors.len() >= MAX_ERRORS
    }

    fn skip_to_newline(&mut self) {
        while !matches!(self.peek(), TokenKind::Newline | TokenKind::Eof | TokenKind::Dedent) {
            self.advance();
        }
        if matches!(self.peek(), TokenKind::Newline) {
            self.advance();
        }
    }

    fn is_toplevel_keyword(kw: Keyword) -> bool {
        matches!(kw, Keyword::Agent | Keyword::Constraints | Keyword::Test
            | Keyword::Flow | Keyword::System | Keyword::Import)
    }

    fn skip_to_toplevel(&mut self) {
        loop {
            match self.peek() {
                TokenKind::Eof => break,
                TokenKind::Keyword(kw) if Self::is_toplevel_keyword(*kw) => break,
                _ => { self.advance(); }
            }
        }
    }
}

// --- Top-level dispatch ---

impl<'a> Parser<'a> {
    fn parse_file(&mut self) -> HmnFile {
        let mut file = HmnFile::default();

        loop {
            self.skip_newlines();
            if self.at_eof() || self.too_many_errors() {
                break;
            }

            match self.peek().clone() {
                TokenKind::Keyword(Keyword::Import) => {
                    if let Some(imp) = self.parse_import() {
                        file.imports.push(imp);
                    }
                }
                TokenKind::Keyword(Keyword::Agent) => {
                    if file.agent.is_some() {
                        self.error("duplicate AGENT declaration");
                        self.skip_to_toplevel();
                        continue;
                    }
                    if let Some(agent) = self.parse_agent() {
                        file.agent = Some(agent);
                    }
                }
                TokenKind::Keyword(Keyword::System) => {
                    match file.agent.as_mut() {
                        Some(agent) => {
                            if agent.system.is_some() {
                                self.error("duplicate SYSTEM declaration");
                                self.skip_to_newline();
                            } else if let Some(sys) = self.parse_system() {
                                agent.system = Some(sys);
                            }
                        }
                        None => {
                            self.error("SYSTEM without preceding AGENT declaration");
                            self.skip_to_newline();
                        }
                    }
                }
                TokenKind::Keyword(Keyword::Constraints) => {
                    if let Some(block) = self.parse_constraints() {
                        file.constraints.push(block);
                    }
                }
                TokenKind::Keyword(Keyword::Flow) => {
                    if let Some(block) = self.parse_flow() {
                        file.flows.push(block);
                    }
                }
                TokenKind::Keyword(Keyword::Test) => {
                    if let Some(block) = self.parse_test() {
                        file.tests.push(block);
                    }
                }
                _ => {
                    self.error(format!("unexpected token: {}", self.peek()));
                    self.skip_to_toplevel();
                }
            }
        }

        file
    }
}

// --- Block parsers ---

impl<'a> Parser<'a> {
    fn parse_import(&mut self) -> Option<Import> {
        let span = self.advance().span; // consume IMPORT
        let target = match self.peek() {
            TokenKind::Path(p) => {
                let t = ImportTarget::Path(p.to_string());
                self.advance();
                t
            }
            TokenKind::Package(pkg) => {
                let t = ImportTarget::Package(pkg.to_string());
                self.advance();
                t
            }
            _ => {
                self.error("expected file path or package name after IMPORT");
                self.skip_to_newline();
                return None;
            }
        };
        self.expect_newline();
        Some(Import { target, span })
    }

    fn parse_agent(&mut self) -> Option<AgentDecl> {
        let span = self.advance().span; // consume AGENT
        let name = match self.peek() {
            TokenKind::Ident(id) => {
                let n = id.to_string();
                self.advance();
                n
            }
            _ => {
                self.error("expected identifier after AGENT");
                self.skip_to_newline();
                return None;
            }
        };
        self.expect_newline();
        self.skip_newlines();

        let mut agent = AgentDecl {
            name,
            properties: Vec::new(),
            system: None,
            span,
        };

        // Optional indented body
        if matches!(self.peek(), TokenKind::Indent) {
            self.advance();
            loop {
                self.skip_newlines();
                match self.peek() {
                    TokenKind::Dedent => { self.advance(); break; }
                    TokenKind::Eof => break,
                    TokenKind::Keyword(Keyword::System) => {
                        if agent.system.is_some() {
                            self.error("duplicate SYSTEM declaration");
                            self.skip_to_newline();
                        } else if let Some(sys) = self.parse_system() {
                            agent.system = Some(sys);
                        }
                    }
                    TokenKind::Ident(_) => {
                        if let Some(prop) = self.parse_property() {
                            agent.properties.push(prop);
                        }
                    }
                    _ => {
                        self.error(format!("unexpected token in AGENT body: {}", self.peek()));
                        self.skip_to_newline();
                    }
                }
                if self.too_many_errors() { break; }
            }
        }

        Some(agent)
    }

    fn parse_system(&mut self) -> Option<SystemDecl> {
        let span = self.advance().span; // consume SYSTEM
        let path = match self.peek() {
            TokenKind::Path(p) => {
                let s = p.to_string();
                self.advance();
                s
            }
            _ => {
                self.error("expected file path after SYSTEM");
                self.skip_to_newline();
                return None;
            }
        };
        self.expect_newline();
        Some(SystemDecl { path, span })
    }

    fn parse_property(&mut self) -> Option<Property> {
        let span = self.span();
        let key = match self.peek() {
            TokenKind::Ident(id) => {
                let k = id.to_string();
                self.advance();
                k
            }
            _ => {
                self.error("expected property name");
                self.skip_to_newline();
                return None;
            }
        };

        if !matches!(self.peek(), TokenKind::Equals) {
            self.error("expected '=' after property name");
            self.skip_to_newline();
            return None;
        }
        self.advance(); // consume =

        let value = match self.peek() {
            TokenKind::Str(s) => {
                let v = Value::Str(s.clone());
                self.advance();
                v
            }
            TokenKind::Number(n) => {
                let v = Value::Number(*n);
                self.advance();
                v
            }
            TokenKind::Bool(b) => {
                let v = Value::Bool(*b);
                self.advance();
                v
            }
            TokenKind::Path(p) => {
                let v = Value::Path(p.to_string());
                self.advance();
                v
            }
            _ => {
                self.error("expected value after '='");
                self.skip_to_newline();
                return None;
            }
        };

        self.expect_newline();
        Some(Property { key, value, span })
    }

    fn parse_constraints(&mut self) -> Option<ConstraintsBlock> {
        let span = self.advance().span; // consume CONSTRAINTS
        let name = match self.peek() {
            TokenKind::Ident(id) => {
                let n = id.to_string();
                self.advance();
                n
            }
            _ => {
                self.error("expected identifier after CONSTRAINTS");
                self.skip_to_newline();
                return None;
            }
        };
        self.expect_newline();
        self.skip_newlines();

        let mut constraints = Vec::new();

        if !self.expect_indent() {
            return Some(ConstraintsBlock { name, constraints, span });
        }

        loop {
            self.skip_newlines();
            match self.peek() {
                TokenKind::Dedent => { self.advance(); break; }
                TokenKind::Eof => break,
                TokenKind::Keyword(kw) if is_constraint_keyword(*kw) => {
                    if let Some(c) = self.parse_constraint() {
                        constraints.push(c);
                    }
                }
                _ => {
                    self.error(format!("expected constraint keyword (NEVER/MUST/SHOULD/AVOID/MAY), got: {}", self.peek()));
                    self.skip_to_newline();
                }
            }
            if self.too_many_errors() { break; }
        }

        Some(ConstraintsBlock { name, constraints, span })
    }

    fn parse_constraint(&mut self) -> Option<Constraint> {
        let span = self.span();
        let level = match self.peek() {
            TokenKind::Keyword(Keyword::Never) => ConstraintLevel::Never,
            TokenKind::Keyword(Keyword::Must) => ConstraintLevel::Must,
            TokenKind::Keyword(Keyword::Should) => ConstraintLevel::Should,
            TokenKind::Keyword(Keyword::Avoid) => ConstraintLevel::Avoid,
            TokenKind::Keyword(Keyword::May) => ConstraintLevel::May,
            _ => {
                self.error("expected constraint level keyword");
                self.skip_to_newline();
                return None;
            }
        };
        self.advance(); // consume level keyword

        let text = match self.peek() {
            TokenKind::Text(t) => {
                let s = t.to_string();
                self.advance();
                s
            }
            _ => {
                self.error("expected constraint text after level keyword");
                self.skip_to_newline();
                return None;
            }
        };

        self.expect_newline();
        Some(Constraint { level, text, span })
    }

    fn parse_flow(&mut self) -> Option<FlowBlock> {
        let span = self.advance().span; // consume FLOW
        let name = match self.peek() {
            TokenKind::Ident(id) => {
                let n = id.to_string();
                self.advance();
                n
            }
            _ => {
                self.error("expected identifier after FLOW");
                self.skip_to_newline();
                return None;
            }
        };
        self.expect_newline();
        self.skip_newlines();

        let mut steps = Vec::new();

        if !self.expect_indent() {
            return Some(FlowBlock { name, steps, span });
        }

        loop {
            self.skip_newlines();
            match self.peek() {
                TokenKind::Dedent => { self.advance(); break; }
                TokenKind::Eof => break,
                _ => {
                    if let Some(step) = self.parse_flow_step() {
                        steps.push(step);
                    }
                }
            }
            if self.too_many_errors() { break; }
        }

        Some(FlowBlock { name, steps, span })
    }

    fn parse_flow_step(&mut self) -> Option<FlowStep> {
        let span = self.span();
        let mut words = Vec::new();

        loop {
            match self.peek() {
                TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof => break,
                TokenKind::Ident(id) => { words.push(id.to_string()); self.advance(); }
                TokenKind::Text(t) => { words.push(t.to_string()); self.advance(); }
                TokenKind::Keyword(kw) => { words.push(kw.to_string()); self.advance(); }
                TokenKind::Number(n) => { words.push(n.to_string()); self.advance(); }
                TokenKind::Bool(b) => { words.push(b.to_string()); self.advance(); }
                _ => { words.push(format!("{}", self.peek())); self.advance(); }
            }
        }

        if words.is_empty() {
            self.error("expected flow step text");
            self.skip_to_newline();
            return None;
        }

        self.expect_newline();
        Some(FlowStep { text: words.join(" "), span })
    }

    fn parse_test(&mut self) -> Option<TestBlock> {
        let span = self.advance().span; // consume TEST
        self.expect_newline();
        self.skip_newlines();

        let mut inputs = Vec::new();
        let mut expects = Vec::new();

        if !self.expect_indent() {
            return Some(TestBlock { inputs, expects, span });
        }

        loop {
            self.skip_newlines();
            match self.peek() {
                TokenKind::Dedent => { self.advance(); break; }
                TokenKind::Eof => break,
                TokenKind::Keyword(Keyword::Input) => {
                    if let Some(inp) = self.parse_test_input() {
                        inputs.push(inp);
                    }
                }
                TokenKind::Keyword(Keyword::Expect) => {
                    match self.parse_test_expect() {
                        Ok(exp) => expects.push(exp),
                        Err(()) => {} // error already recorded, line skipped
                    }
                }
                _ => {
                    self.error(format!("expected INPUT or EXPECT in TEST block, got: {}", self.peek()));
                    self.skip_to_newline();
                }
            }
            if self.too_many_errors() { break; }
        }

        Some(TestBlock { inputs, expects, span })
    }

    fn parse_test_input(&mut self) -> Option<TestInput> {
        let span = self.advance().span; // consume INPUT
        match self.peek() {
            TokenKind::Str(s) => {
                let value = s.clone();
                self.advance();
                self.expect_newline();
                Some(TestInput { value, span })
            }
            _ => {
                self.error("expected quoted string after INPUT");
                self.skip_to_newline();
                None
            }
        }
    }

    fn parse_test_expect(&mut self) -> Result<TestExpect, ()> {
        let span = self.advance().span; // consume EXPECT

        let negated = if matches!(self.peek(), TokenKind::Keyword(Keyword::Not)) {
            self.advance();
            true
        } else {
            false
        };

        let op = match self.peek() {
            TokenKind::Keyword(Keyword::Contains) => {
                self.advance();
                TestOp::Contains
            }
            TokenKind::Keyword(Keyword::Matches) => {
                self.advance();
                TestOp::Matches
            }
            _ => {
                self.error("unsupported EXPECT form");
                self.skip_to_newline();
                return Err(());
            }
        };

        let value = match self.peek() {
            TokenKind::Str(s) => {
                let v = s.clone();
                self.advance();
                v
            }
            _ => {
                self.error("expected quoted string after CONTAINS/MATCHES");
                self.skip_to_newline();
                return Err(());
            }
        };

        self.expect_newline();
        Ok(TestExpect { negated, op, value, span })
    }
}
