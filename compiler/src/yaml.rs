use crate::ir::CompiledOutput;

pub fn emit_yaml(compiled: &CompiledOutput) -> String {
    serde_yaml::to_string(compiled).expect("serialization cannot fail")
}
