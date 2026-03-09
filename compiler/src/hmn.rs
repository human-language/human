use human_parser::{HmnFile, ImportTarget, TestOp};
use human_resolver::Resolved;
use crate::util::{level_str, format_value_hmn};

pub fn emit_hmn(resolved: &Resolved) -> String {
    let mut out = String::new();

    out.push_str("AGENT ");
    out.push_str(&resolved.agent.name);
    out.push('\n');

    for prop in &resolved.agent.properties {
        out.push_str("  ");
        out.push_str(&prop.key);
        out.push_str(" = ");
        out.push_str(&format_value_hmn(&prop.value));
        out.push('\n');
    }

    if let Some(sys) = &resolved.agent.system {
        out.push('\n');
        out.push_str("SYSTEM ");
        out.push_str(&sys.path);
        out.push('\n');
    }

    for block in &resolved.constraints {
        out.push('\n');
        out.push_str("CONSTRAINTS ");
        out.push_str(&block.name);
        out.push('\n');
        for c in &block.constraints {
            out.push_str("  ");
            out.push_str(level_str(c.level));
            out.push(' ');
            out.push_str(&c.text);
            out.push('\n');
        }
    }

    for block in &resolved.flows {
        out.push('\n');
        out.push_str("FLOW ");
        out.push_str(&block.name);
        out.push('\n');
        for step in &block.steps {
            out.push_str("  ");
            out.push_str(&step.text);
            out.push('\n');
        }
    }

    out
}

/// Re-emit any `.hmn` file -- with or without AGENT, with IMPORT lines preserved.
/// Used by `human fmt`. Unlike `emit_hmn`, this does not require a resolved project.
pub fn emit_file(file: &HmnFile) -> String {
    let mut out = String::new();
    let mut need_blank = false;

    for imp in &file.imports {
        match &imp.target {
            ImportTarget::Path(p) => {
                out.push_str("IMPORT ");
                out.push_str(p);
                out.push('\n');
            }
            ImportTarget::Package(p) => {
                out.push_str("IMPORT @");
                out.push_str(p);
                out.push('\n');
            }
        }
        need_blank = true;
    }

    if let Some(agent) = &file.agent {
        if need_blank {
            out.push('\n');
        }
        out.push_str("AGENT ");
        out.push_str(&agent.name);
        out.push('\n');

        for prop in &agent.properties {
            out.push_str("  ");
            out.push_str(&prop.key);
            out.push_str(" = ");
            out.push_str(&format_value_hmn(&prop.value));
            out.push('\n');
        }

        if let Some(sys) = &agent.system {
            out.push('\n');
            out.push_str("SYSTEM ");
            out.push_str(&sys.path);
            out.push('\n');
        }

        need_blank = true;
    }

    for block in &file.constraints {
        if need_blank {
            out.push('\n');
        }
        out.push_str("CONSTRAINTS ");
        out.push_str(&block.name);
        out.push('\n');
        for c in &block.constraints {
            out.push_str("  ");
            out.push_str(level_str(c.level));
            out.push(' ');
            out.push_str(&c.text);
            out.push('\n');
        }
        need_blank = true;
    }

    for block in &file.flows {
        if need_blank {
            out.push('\n');
        }
        out.push_str("FLOW ");
        out.push_str(&block.name);
        out.push('\n');
        for step in &block.steps {
            out.push_str("  ");
            out.push_str(&step.text);
            out.push('\n');
        }
        need_blank = true;
    }

    for block in &file.tests {
        if need_blank {
            out.push('\n');
        }
        out.push_str("TEST\n");
        for input in &block.inputs {
            out.push_str("  INPUT ");
            let escaped = input.value.replace('\\', "\\\\").replace('"', "\\\"");
            out.push_str(&format!("\"{escaped}\""));
            out.push('\n');
        }
        for expect in &block.expects {
            out.push_str("  EXPECT ");
            if expect.negated {
                out.push_str("NOT ");
            }
            match expect.op {
                TestOp::Contains => out.push_str("CONTAINS "),
                TestOp::Matches => out.push_str("MATCHES "),
            }
            let escaped = expect.value.replace('\\', "\\\\").replace('"', "\\\"");
            out.push_str(&format!("\"{escaped}\""));
            out.push('\n');
        }
        need_blank = true;
    }

    out
}
