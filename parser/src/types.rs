use human_lexer::Span;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct HmnFile {
    pub imports: Vec<Import>,
    pub agent: Option<AgentDecl>,
    pub constraints: Vec<ConstraintsBlock>,
    pub flows: Vec<FlowBlock>,
    pub tests: Vec<TestBlock>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub target: ImportTarget,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportTarget {
    Path(String),
    Package(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentDecl {
    pub name: String,
    pub properties: Vec<Property>,
    pub system: Option<SystemDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemDecl {
    pub path: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub key: String,
    pub value: Value,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    Number(f64),
    Bool(bool),
    Path(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstraintsBlock {
    pub name: String,
    pub constraints: Vec<Constraint>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Constraint {
    pub level: ConstraintLevel,
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintLevel {
    Never,
    Must,
    Should,
    Avoid,
    May,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowBlock {
    pub name: String,
    pub steps: Vec<FlowStep>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowStep {
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestBlock {
    pub inputs: Vec<TestInput>,
    pub expects: Vec<TestExpect>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestInput {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestExpect {
    pub negated: bool,
    pub op: TestOp,
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestOp {
    Contains,
    Matches,
}
