use std::path::Path;
use crate::error::CompileError;

pub fn load_system_prompt(
    system_path: &str,
    root_file: &Path,
) -> Result<String, CompileError> {
    let base_dir = root_file.parent().unwrap_or(Path::new("."));
    let resolved = base_dir.join(system_path);

    std::fs::read_to_string(&resolved).map_err(|e| {
        let msg = match e.kind() {
            std::io::ErrorKind::NotFound => "file not found".to_string(),
            std::io::ErrorKind::InvalidData => "file is not valid UTF-8".to_string(),
            _ => format!("cannot read file: {e}"),
        };
        CompileError::new(&resolved, msg)
    })
}
