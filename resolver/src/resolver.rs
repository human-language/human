use std::collections::HashMap;
use std::path::{Path, PathBuf};
use human_lexer::Lexer;
use human_parser::{HmnFile, parse};
use crate::error::{ResolveError, short_name};
use crate::resolve_path::resolve_import_path;

pub struct Resolver {
    pub project_root: PathBuf,
    pub file_cache: HashMap<PathBuf, HmnFile>,
    pub adjacency: HashMap<PathBuf, Vec<PathBuf>>,
    pub errors: Vec<ResolveError>,
}

impl Resolver {
    pub fn new(project_root: PathBuf) -> Self {
        Resolver {
            project_root,
            file_cache: HashMap::new(),
            adjacency: HashMap::new(),
            errors: Vec::new(),
        }
    }

    pub fn load_file(&self, path: &Path) -> Result<HmnFile, Vec<ResolveError>> {
        let bytes = std::fs::read(path).map_err(|e| {
            vec![ResolveError::at_file(path, format!("cannot read file: {}", e))]
        })?;

        let tokens = Lexer::new(&bytes).tokenize().map_err(|lex_errors| {
            lex_errors.iter().map(|e| ResolveError::from_lex_error(path, e)).collect::<Vec<_>>()
        })?;

        let hmn = parse(&tokens).map_err(|parse_errors| {
            parse_errors.iter().map(|e| ResolveError::from_parse_error(path, e)).collect::<Vec<_>>()
        })?;

        Ok(hmn)
    }

    pub fn resolve_recursive(&mut self, path: &Path, stack: &mut Vec<PathBuf>) {
        let canonical = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(_) => {
                self.errors.push(ResolveError::at_file(
                    path,
                    format!("file not found: {}", path.display()),
                ));
                return;
            }
        };

        if let Some(pos) = stack.iter().position(|p| p == &canonical) {
            let chain: Vec<String> = stack[pos..]
                .iter()
                .chain(std::iter::once(&canonical))
                .map(|p| short_name(p))
                .collect();
            self.errors.push(ResolveError::at_file(
                &canonical,
                format!("circular import detected: {}", chain.join(" -> ")),
            ));
            return;
        }

        if self.file_cache.contains_key(&canonical) {
            return;
        }

        let hmn = match self.load_file(&canonical) {
            Ok(hmn) => hmn,
            Err(errs) => {
                self.errors.extend(errs);
                return;
            }
        };

        let imports: Vec<_> = hmn.imports.clone();
        self.file_cache.insert(canonical.clone(), hmn);

        stack.push(canonical.clone());

        let mut deps = Vec::new();
        for import in &imports {
            match resolve_import_path(
                &import.target,
                &canonical,
                &self.project_root,
                import.span,
            ) {
                Ok(resolved) => {
                    deps.push(resolved.clone());
                    self.resolve_recursive(&resolved, stack);
                }
                Err(e) => {
                    self.errors.push(e);
                }
            }
        }

        self.adjacency.insert(canonical.clone(), deps);
        stack.pop();
    }

    pub fn topo_sort(&self) -> Result<Vec<PathBuf>, Vec<ResolveError>> {
        let mut in_degree: HashMap<PathBuf, usize> = HashMap::new();

        for path in self.file_cache.keys() {
            in_degree.entry(path.clone()).or_insert(0);
        }
        for (_, deps) in &self.adjacency {
            for dep in deps {
                *in_degree.entry(dep.clone()).or_insert(0) += 1;
            }
        }

        let mut queue: std::collections::BinaryHeap<PathBuf> = in_degree
            .iter()
            .filter(|&(_, deg)| *deg == 0)
            .map(|(p, _)| p.clone())
            .collect();

        let mut result = Vec::new();

        while let Some(node) = queue.pop() {
            if let Some(deps) = self.adjacency.get(&node) {
                for dep in deps {
                    if let Some(deg) = in_degree.get_mut(dep) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push(dep.clone());
                        }
                    }
                }
            }
            result.push(node);
        }

        if result.len() != self.file_cache.len() {
            return Err(vec![ResolveError::at_file(
                self.project_root.join("main.hmn"),
                "internal error: topological sort failed (cycle should have been caught earlier)",
            )]);
        }

        // Reverse: deepest dependencies first, root last
        result.reverse();
        Ok(result)
    }
}

