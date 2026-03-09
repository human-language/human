pub mod error;
pub mod types;
pub mod resolve_path;
pub mod resolver;
pub mod merge;

#[cfg(test)]
mod tests;

use std::path::Path;
use crate::error::ResolveError;
use crate::types::ResolvedFile;
use crate::resolver::Resolver;
use crate::merge::merge;

pub use error::ResolveError as Error;
pub use types::ResolvedFile as Resolved;

pub fn resolve(root: &Path, project_root: &Path) -> Result<ResolvedFile, Vec<ResolveError>> {
    let canonical_root = std::fs::canonicalize(root).map_err(|_| {
        vec![ResolveError::at_file(root, format!("file not found: {}", root.display()))]
    })?;

    let canonical_project = std::fs::canonicalize(project_root).map_err(|_| {
        vec![ResolveError::at_file(
            project_root,
            format!("project root not found: {}", project_root.display()),
        )]
    })?;

    let mut resolver = Resolver::new(canonical_project);
    let mut stack = Vec::new();
    resolver.resolve_recursive(&canonical_root, &mut stack);

    if !resolver.errors.is_empty() {
        return Err(resolver.errors);
    }

    let sorted = resolver.topo_sort()?;
    merge(&canonical_root, &sorted, &resolver.file_cache)
}
