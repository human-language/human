use std::path::{Path, PathBuf};
use human_lexer::Span;
use human_parser::ImportTarget;
use crate::error::ResolveError;

pub fn resolve_import_path(
    target: &ImportTarget,
    importing_file: &Path,
    project_root: &Path,
    span: Span,
) -> Result<PathBuf, ResolveError> {
    let raw_path = match target {
        ImportTarget::Path(p) => {
            let base = importing_file.parent().unwrap_or(Path::new("."));
            base.join(p)
        }
        ImportTarget::Package(pkg) => {
            match pkg.split_once('/') {
                None => project_root.join("human_modules").join(pkg).join("main.hmn"),
                Some((name, rest)) => {
                    let mut p = project_root.join("human_modules").join(name).join(rest);
                    if p.extension().is_none() {
                        p.set_extension("hmn");
                    }
                    p
                }
            }
        }
    };

    let canonical = std::fs::canonicalize(&raw_path).map_err(|_| {
        ResolveError::at_span(
            importing_file,
            span,
            format!(
                "file not found: {} (resolved to {})",
                target_display(target),
                raw_path.display()
            ),
        )
    })?;

    if !canonical.starts_with(project_root) {
        return Err(ResolveError::at_span(
            importing_file,
            span,
            format!(
                "import path escapes project root: {} (resolved to {})",
                target_display(target),
                canonical.display()
            ),
        ));
    }

    Ok(canonical)
}

fn target_display(target: &ImportTarget) -> String {
    match target {
        ImportTarget::Path(p) => p.clone(),
        ImportTarget::Package(p) => p.clone(),
    }
}
