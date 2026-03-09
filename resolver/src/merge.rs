use std::collections::HashMap;
use std::path::{Path, PathBuf};
use human_parser::HmnFile;
use crate::error::{ResolveError, short_name};
use crate::types::ResolvedFile;

pub fn merge(
    root_path: &Path,
    sorted_paths: &[PathBuf],
    file_cache: &HashMap<PathBuf, HmnFile>,
) -> Result<ResolvedFile, Vec<ResolveError>> {
    let mut errors = Vec::new();
    let mut agent = None;
    let mut constraints = Vec::new();
    let mut flows = Vec::new();
    let mut tests = Vec::new();
    let mut constraint_names: HashMap<String, PathBuf> = HashMap::new();
    let mut flow_names: HashMap<String, PathBuf> = HashMap::new();

    for path in sorted_paths {
        let hmn = match file_cache.get(path) {
            Some(h) => h,
            None => continue,
        };

        if let Some(ref ag) = hmn.agent {
            if path != root_path {
                errors.push(ResolveError::at_span(
                    path,
                    ag.span,
                    "AGENT can only appear in main.hmn",
                ));
            } else {
                agent = Some(ag.clone());
            }
        }

        for block in &hmn.constraints {
            if let Some(prev) = constraint_names.get(&block.name) {
                errors.push(ResolveError::at_span(
                    path,
                    block.span,
                    format!(
                        "duplicate CONSTRAINTS block '{}' -- also defined in {}",
                        block.name,
                        short_name(prev)
                    ),
                ));
            } else {
                constraint_names.insert(block.name.clone(), path.clone());
                constraints.push(block.clone());
            }
        }

        for block in &hmn.flows {
            if let Some(prev) = flow_names.get(&block.name) {
                errors.push(ResolveError::at_span(
                    path,
                    block.span,
                    format!(
                        "duplicate FLOW block '{}' -- also defined in {}",
                        block.name,
                        short_name(prev)
                    ),
                ));
            } else {
                flow_names.insert(block.name.clone(), path.clone());
                flows.push(block.clone());
            }
        }

        tests.extend(hmn.tests.iter().cloned());
    }

    if agent.is_none() {
        errors.push(ResolveError::at_file(root_path, "no AGENT declaration found"));
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(ResolvedFile {
        agent: agent.unwrap(),
        constraints,
        flows,
        tests,
        sources: sorted_paths.to_vec(),
    })
}

