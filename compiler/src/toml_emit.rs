use crate::ir::CompiledOutput;

pub fn emit_toml(compiled: &CompiledOutput) -> String {
    toml::to_string_pretty(compiled).expect("serialization cannot fail")
}
