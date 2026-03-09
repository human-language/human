use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub offset: u32,
    pub len: u16,
    pub line: u32,
    pub col: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind<'a> {
    Keyword(Keyword),
    Ident(&'a str),
    Text(&'a str),
    Str(String),
    Number(f64),
    Bool(bool),
    Path(&'a str),
    Package(&'a str),
    Equals,
    Comment(&'a str),
    Indent,
    Dedent,
    Newline,
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Agent,
    Constraints,
    Test,
    Flow,
    System,
    Import,
    Never,
    Must,
    Should,
    Avoid,
    May,
    Input,
    Expect,
    Not,
    Contains,
    Matches,
}

pub fn keyword_from_str(s: &str) -> Option<Keyword> {
    match s {
        "AGENT" => Some(Keyword::Agent),
        "CONSTRAINTS" => Some(Keyword::Constraints),
        "TEST" => Some(Keyword::Test),
        "FLOW" => Some(Keyword::Flow),
        "SYSTEM" => Some(Keyword::System),
        "IMPORT" => Some(Keyword::Import),
        "NEVER" => Some(Keyword::Never),
        "MUST" => Some(Keyword::Must),
        "SHOULD" => Some(Keyword::Should),
        "AVOID" => Some(Keyword::Avoid),
        "MAY" => Some(Keyword::May),
        "INPUT" => Some(Keyword::Input),
        "EXPECT" => Some(Keyword::Expect),
        "NOT" => Some(Keyword::Not),
        "CONTAINS" => Some(Keyword::Contains),
        "MATCHES" => Some(Keyword::Matches),
        _ => None,
    }
}

pub fn is_constraint_keyword(k: Keyword) -> bool {
    matches!(k, Keyword::Never | Keyword::Must | Keyword::Should | Keyword::Avoid | Keyword::May)
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Keyword::Agent => "AGENT",
            Keyword::Constraints => "CONSTRAINTS",
            Keyword::Test => "TEST",
            Keyword::Flow => "FLOW",
            Keyword::System => "SYSTEM",
            Keyword::Import => "IMPORT",
            Keyword::Never => "NEVER",
            Keyword::Must => "MUST",
            Keyword::Should => "SHOULD",
            Keyword::Avoid => "AVOID",
            Keyword::May => "MAY",
            Keyword::Input => "INPUT",
            Keyword::Expect => "EXPECT",
            Keyword::Not => "NOT",
            Keyword::Contains => "CONTAINS",
            Keyword::Matches => "MATCHES",
        };
        write!(f, "{}", s)
    }
}

impl<'a> fmt::Display for TokenKind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Keyword(k) => write!(f, "{}", k),
            TokenKind::Ident(s) => write!(f, "{}", s),
            TokenKind::Text(s) => write!(f, "\"{}\"", s),
            TokenKind::Str(s) => write!(f, "\"{}\"", s),
            TokenKind::Number(n) => write!(f, "{}", n),
            TokenKind::Bool(b) => write!(f, "{}", b),
            TokenKind::Path(p) => write!(f, "{}", p),
            TokenKind::Package(p) => write!(f, "{}", p),
            TokenKind::Equals => write!(f, "="),
            TokenKind::Comment(c) => write!(f, "# {}", c),
            TokenKind::Indent => write!(f, "INDENT"),
            TokenKind::Dedent => write!(f, "DEDENT"),
            TokenKind::Newline => write!(f, "NEWLINE"),
            TokenKind::Eof => write!(f, "EOF"),
        }
    }
}
