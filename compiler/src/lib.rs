pub mod error;
pub mod system;
pub mod util;
pub mod ir;
pub mod hmn;
pub mod prompt;
pub mod json;
pub mod yaml;
pub mod toml_emit;
pub mod txt;

#[cfg(test)]
mod tests;

use std::path::Path;
use human_resolver::Resolved;
use crate::error::CompileError;
use crate::system::load_system_prompt;

pub use error::CompileError as Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Hmn,
    Prompt,
    Json,
    Yaml,
    Toml,
    Txt,
}

pub fn compile(
    resolved: &Resolved,
    root_file: &Path,
    format: OutputFormat,
) -> Result<String, CompileError> {
    if format == OutputFormat::Hmn {
        return Ok(hmn::emit_hmn(resolved));
    }

    let system_content = match &resolved.agent.system {
        Some(sys) => Some(load_system_prompt(&sys.path, root_file)?),
        None => None,
    };

    match format {
        OutputFormat::Hmn => unreachable!(),
        OutputFormat::Prompt => {
            Ok(prompt::emit_prompt(resolved, system_content.as_deref()))
        }
        OutputFormat::Txt => {
            Ok(txt::emit_txt(resolved, system_content.as_deref()))
        }
        OutputFormat::Json => {
            let compiled = ir::build(resolved, system_content.as_deref());
            Ok(json::emit_json(&compiled))
        }
        OutputFormat::Yaml => {
            let compiled = ir::build(resolved, system_content.as_deref());
            Ok(yaml::emit_yaml(&compiled))
        }
        OutputFormat::Toml => {
            let compiled = ir::build(resolved, system_content.as_deref());
            Ok(toml_emit::emit_toml(&compiled))
        }
    }
}

pub fn compile_hmn(resolved: &Resolved) -> String {
    hmn::emit_hmn(resolved)
}

pub fn compile_prompt(resolved: &Resolved, root_file: &Path) -> Result<String, CompileError> {
    compile(resolved, root_file, OutputFormat::Prompt)
}

pub fn compile_json(resolved: &Resolved, root_file: &Path) -> Result<String, CompileError> {
    compile(resolved, root_file, OutputFormat::Json)
}

pub fn compile_yaml(resolved: &Resolved, root_file: &Path) -> Result<String, CompileError> {
    compile(resolved, root_file, OutputFormat::Yaml)
}

pub fn compile_toml(resolved: &Resolved, root_file: &Path) -> Result<String, CompileError> {
    compile(resolved, root_file, OutputFormat::Toml)
}

pub fn compile_txt(resolved: &Resolved, root_file: &Path) -> Result<String, CompileError> {
    compile(resolved, root_file, OutputFormat::Txt)
}
