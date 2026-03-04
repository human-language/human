use crate::error::LexError;
use crate::token::{Token, TokenKind, Keyword, Span, keyword_from_str, is_constraint_keyword};

const MAX_ERRORS: usize = 10;

pub struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
    line: u32,
    col: u16,
    indent_stack: Vec<u16>,
    tokens: Vec<Token<'a>>,
    errors: Vec<LexError>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a [u8]) -> Self {
        Lexer {
            src,
            pos: 0,
            line: 1,
            col: 1,
            indent_stack: vec![0],
            tokens: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token<'a>>, Vec<LexError>> {
        self.validate_ascii();
        if self.errors.len() >= MAX_ERRORS {
            return Err(self.errors);
        }

        while self.pos < self.src.len() {
            self.lex_line();
            if self.errors.len() >= MAX_ERRORS {
                break;
            }
        }

        self.drain_indents();
        self.emit(TokenKind::Eof, self.pos, 0);

        if self.errors.is_empty() {
            Ok(self.tokens)
        } else {
            Err(self.errors)
        }
    }

    fn validate_ascii(&mut self) {
        let mut i = 0;
        let mut line: u32 = 1;
        let mut col: u16 = 1;
        while i < self.src.len() {
            let b = self.src[i];
            if b == b'\r' {
                i += 1;
                col = 1;
                if i < self.src.len() && self.src[i] == b'\n' {
                    // CRLF — skip, the \n will be handled next
                } else {
                    line += 1;
                }
                continue;
            }
            if b == b'\n' {
                line += 1;
                col = 1;
                i += 1;
                continue;
            }
            if b == b'\t' || (b >= 0x20 && b <= 0x7E) {
                col += 1;
                i += 1;
                continue;
            }
            self.errors.push(LexError::new(
                line, col,
                format!("invalid character U+{:04X} -- .hmn files must be ASCII only", b as u32),
            ));
            if self.errors.len() >= MAX_ERRORS {
                return;
            }
            i += 1;
            col += 1;
        }
    }

    fn lex_line(&mut self) {
        // Count leading whitespace, reject tabs
        let mut spaces: u16 = 0;
        let mut has_tab = false;
        while self.pos < self.src.len() {
            match self.src[self.pos] {
                b' ' => {
                    spaces += 1;
                    self.pos += 1;
                }
                b'\t' => {
                    if !has_tab {
                        self.errors.push(LexError::new(
                            self.line, spaces + 1,
                            "tabs not allowed for indentation -- use 2 spaces",
                        ));
                        has_tab = true;
                    }
                    spaces += 1; // count tab as 1 for error recovery
                    self.pos += 1;
                }
                _ => break,
            }
        }

        // Check for blank line or end of input
        if self.at_eol() {
            self.finish_line();
            return;
        }

        // Validate indentation is multiple of 2
        if !has_tab && spaces % 2 != 0 {
            self.errors.push(LexError::new(
                self.line, 1,
                "indentation must be a multiple of 2 spaces",
            ));
        }

        // Emit INDENT/DEDENT
        if !has_tab {
            self.update_indent(spaces);
        }

        self.col = spaces + 1;

        // Check for comment line: # as first non-whitespace
        if self.pos < self.src.len() && self.src[self.pos] == b'#' {
            self.lex_comment();
            self.finish_line();
            return;
        }

        // Lex tokens on this line
        self.lex_line_content();
        self.finish_line();
    }

    fn update_indent(&mut self, spaces: u16) {
        let current = *self.indent_stack.last().unwrap();
        if spaces > current {
            self.indent_stack.push(spaces);
            self.emit(TokenKind::Indent, self.pos, 0);
        } else if spaces < current {
            while *self.indent_stack.last().unwrap() > spaces {
                self.indent_stack.pop();
                if *self.indent_stack.last().unwrap() < spaces {
                    self.errors.push(LexError::new(
                        self.line, 1,
                        "indentation does not match any outer level",
                    ));
                    break;
                }
                self.emit(TokenKind::Dedent, self.pos, 0);
            }
        }
    }

    fn drain_indents(&mut self) {
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            self.emit(TokenKind::Dedent, self.pos, 0);
        }
    }

    fn lex_comment(&mut self) {
        let start = self.pos;
        self.pos += 1; // skip #
        // Skip space after #
        if self.pos < self.src.len() && self.src[self.pos] == b' ' {
            self.pos += 1;
        }
        let text_start = self.pos;
        self.skip_to_eol();
        let text_end = self.pos;
        let text = self.str_from(text_start, text_end);
        let text = text.trim_end();
        self.emit(TokenKind::Comment(text), start, (self.pos - start) as u16);
    }

    fn lex_line_content(&mut self) {
        // Lex first token — might be a keyword that triggers modal capture
        let first = self.lex_word_or_token();

        if let Some(TokenKind::Keyword(kw)) = &first {
            if is_constraint_keyword(*kw) {
                // Modal text capture: rest of line is a single Text token
                self.skip_spaces();
                if !self.at_eol() {
                    let text_start = self.pos;
                    self.skip_to_eol();
                    let text = self.str_from(text_start, self.pos).trim_end();
                    if !text.is_empty() {
                        self.emit(TokenKind::Text(text), text_start, (self.pos - text_start) as u16);
                    }
                }
                return;
            }

            if *kw == Keyword::Import {
                // IMPORT has special handling: rest of line is path or ident
                self.skip_spaces();
                if !self.at_eol() {
                    self.lex_import_target();
                }
                return;
            }
        }

        // Normal mode: continue lexing tokens on this line
        while !self.at_eol() && self.errors.len() < MAX_ERRORS {
            self.skip_spaces();
            if self.at_eol() {
                break;
            }
            self.lex_normal_token();
        }
    }

    fn lex_word_or_token(&mut self) -> Option<TokenKind<'a>> {
        self.skip_spaces();
        if self.at_eol() {
            return None;
        }

        let b = self.src[self.pos];
        if is_ident_start(b) {
            let start = self.pos;
            let word = self.consume_ident();
            let kind = if word == "true" {
                TokenKind::Bool(true)
            } else if word == "false" {
                TokenKind::Bool(false)
            } else if let Some(kw) = keyword_from_str(word) {
                TokenKind::Keyword(kw)
            } else {
                TokenKind::Ident(word)
            };
            let len = (self.pos - start) as u16;
            let ret = kind.clone();
            self.emit(kind, start, len);
            Some(ret)
        } else {
            self.lex_normal_token();
            None
        }
    }

    fn lex_import_target(&mut self) {
        let start = self.pos;
        if self.pos + 1 < self.src.len()
            && self.src[self.pos] == b'.'
            && (self.src[self.pos + 1] == b'/' || self.src[self.pos + 1] == b'.')
        {
            // File path
            self.skip_to_eol();
            let path = self.str_from(start, self.pos).trim_end();
            self.emit(TokenKind::Path(path), start, (self.pos - start) as u16);
        } else {
            // Package name: consume until whitespace or EOL
            // Allows a-z, A-Z, 0-9, _, /, -, .
            while self.pos < self.src.len() && !self.at_eol() && self.src[self.pos] != b' ' {
                self.pos += 1;
            }
            let ident = self.str_from(start, self.pos).trim_end();
            self.emit(TokenKind::Ident(ident), start, (self.pos - start) as u16);
        }
    }

    fn lex_normal_token(&mut self) {
        let b = self.src[self.pos];
        match b {
            b'"' => self.lex_string(),
            b'=' => {
                self.emit(TokenKind::Equals, self.pos, 1);
                self.advance();
            }
            b'.' if self.peek_at(1).is_some_and(|c| c == b'/' || c == b'.') => {
                self.lex_path();
            }
            b'-' if self.peek_at(1).is_some_and(|c| c.is_ascii_digit()) => {
                self.lex_number();
            }
            b'0'..=b'9' => {
                self.lex_number();
            }
            _ if is_ident_start(b) => {
                let start = self.pos;
                let word = self.consume_ident();
                let kind = if word == "true" {
                    TokenKind::Bool(true)
                } else if word == "false" {
                    TokenKind::Bool(false)
                } else if let Some(kw) = keyword_from_str(word) {
                    TokenKind::Keyword(kw)
                } else {
                    TokenKind::Ident(word)
                };
                self.emit(kind, start, (self.pos - start) as u16);
            }
            b' ' => {
                self.advance();
            }
            _ => {
                self.errors.push(LexError::new(
                    self.line, self.col,
                    format!("unexpected character '{}'", b as char),
                ));
                self.advance();
            }
        }
    }

    fn lex_string(&mut self) {
        let start = self.pos;
        self.advance(); // skip opening "
        let mut buf = String::new();
        loop {
            if self.pos >= self.src.len() || self.src[self.pos] == b'\n' {
                self.errors.push(LexError::new(
                    self.line, self.col_at(start),
                    "unterminated string literal",
                ));
                break;
            }
            let b = self.src[self.pos];
            if b == b'"' {
                self.advance(); // skip closing "
                break;
            }
            if b == b'\\' && self.pos + 1 < self.src.len() {
                let next = self.src[self.pos + 1];
                match next {
                    b'"' => { buf.push('"'); self.pos += 2; self.col += 2; continue; }
                    b'\\' => { buf.push('\\'); self.pos += 2; self.col += 2; continue; }
                    _ => {}
                }
            }
            buf.push(b as char);
            self.advance();
        }
        self.emit(TokenKind::Str(buf), start, (self.pos - start) as u16);
    }

    fn lex_path(&mut self) {
        let start = self.pos;
        while self.pos < self.src.len() && !self.at_eol() && self.src[self.pos] != b' ' {
            self.pos += 1;
        }
        let path = self.str_from(start, self.pos).trim_end();
        self.col = self.col_at(self.pos);
        self.emit(TokenKind::Path(path), start, (self.pos - start) as u16);
    }

    fn lex_number(&mut self) {
        let start = self.pos;
        if self.src[self.pos] == b'-' {
            self.pos += 1;
        }
        while self.pos < self.src.len() && self.src[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        if self.pos < self.src.len() && self.src[self.pos] == b'.'
            && self.pos + 1 < self.src.len() && self.src[self.pos + 1].is_ascii_digit()
        {
            self.pos += 1; // skip .
            while self.pos < self.src.len() && self.src[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }
        let num_str = self.str_from(start, self.pos);
        match num_str.parse::<f64>() {
            Ok(n) => self.emit(TokenKind::Number(n), start, (self.pos - start) as u16),
            Err(_) => {
                self.errors.push(LexError::new(
                    self.line, self.col_at(start),
                    format!("invalid number literal '{}'", num_str),
                ));
            }
        }
        self.col = self.col_at(self.pos);
    }

    fn consume_ident(&mut self) -> &'a str {
        let start = self.pos;
        while self.pos < self.src.len() && is_ident_cont(self.src[self.pos]) {
            self.pos += 1;
        }
        self.col = self.col_at(self.pos);
        self.str_from(start, self.pos)
    }

    // --- helpers ---

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

    fn finish_line(&mut self) {
        let has_newline = self.pos < self.src.len()
            && (self.src[self.pos] == b'\n' || self.src[self.pos] == b'\r');
        if has_newline {
            self.emit(TokenKind::Newline, self.pos, 0);
        }
        self.skip_newline();
    }

    fn advance(&mut self) {
        if self.pos < self.src.len() {
            self.pos += 1;
            self.col += 1;
        }
    }

    fn skip_spaces(&mut self) {
        while self.pos < self.src.len() && self.src[self.pos] == b' ' {
            self.pos += 1;
            self.col += 1;
        }
    }

    fn skip_to_eol(&mut self) {
        while self.pos < self.src.len() && self.src[self.pos] != b'\n' && self.src[self.pos] != b'\r' {
            self.pos += 1;
        }
        self.col = self.col_at(self.pos);
    }

    fn skip_newline(&mut self) {
        if self.pos < self.src.len() {
            if self.src[self.pos] == b'\r' {
                self.pos += 1;
                if self.pos < self.src.len() && self.src[self.pos] == b'\n' {
                    self.pos += 1;
                }
            } else if self.src[self.pos] == b'\n' {
                self.pos += 1;
            }
        }
        self.line += 1;
        self.col = 1;
    }

    fn at_eol(&self) -> bool {
        self.pos >= self.src.len()
            || self.src[self.pos] == b'\n'
            || self.src[self.pos] == b'\r'
    }

    fn peek_at(&self, offset: usize) -> Option<u8> {
        self.src.get(self.pos + offset).copied()
    }

    fn str_from(&self, start: usize, end: usize) -> &'a str {
        // Safe: we validated ASCII in the pre-pass
        unsafe { std::str::from_utf8_unchecked(&self.src[start..end]) }
    }

    fn col_at(&self, pos: usize) -> u16 {
        // Walk back to find start of current line
        let mut p = pos;
        while p > 0 && self.src[p - 1] != b'\n' {
            p -= 1;
        }
        (pos - p + 1) as u16
    }
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_ident_cont(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}
