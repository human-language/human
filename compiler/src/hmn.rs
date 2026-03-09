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
