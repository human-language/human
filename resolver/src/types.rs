use std::path::PathBuf;
use human_parser::{AgentDecl, ConstraintsBlock, FlowBlock, TestBlock};

#[derive(Debug, Clone)]
pub struct ResolvedFile {
    pub agent: AgentDecl,
    pub constraints: Vec<ConstraintsBlock>,
    pub flows: Vec<FlowBlock>,
    pub tests: Vec<TestBlock>,
    pub sources: Vec<PathBuf>,
}
