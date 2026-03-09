use crate::ir::CompiledOutput;

pub fn emit_json(compiled: &CompiledOutput) -> String {
    serde_json::to_string_pretty(compiled).expect("serialization cannot fail")
}
