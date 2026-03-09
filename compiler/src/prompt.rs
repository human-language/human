use human_resolver::Resolved;
use crate::util::{level_str, format_value_bare};

pub fn emit_prompt(resolved: &Resolved, system_content: Option<&str>) -> String {
    let mut out = String::new();

    if let Some(sys) = system_content {
        if !sys.is_empty() {
            out.push_str(sys.trim_end());
            out.push_str("\n\n---\n\n");
        }
    }

    out.push_str("# ");
    out.push_str(&resolved.agent.name);
    out.push('\n');

    for prop in &resolved.agent.properties {
        out.push_str(&prop.key);
        out.push_str(" = ");
        out.push_str(&format_value_bare(&prop.value));
        out.push('\n');
    }

    for block in &resolved.constraints {
        out.push('\n');
        out.push_str("## ");
        out.push_str(&block.name);
        out.push('\n');
        for c in &block.constraints {
            out.push_str("- ");
            out.push_str(level_str(c.level));
            out.push_str(": ");
            out.push_str(&c.text);
            out.push('\n');
        }
    }

    for block in &resolved.flows {
        out.push('\n');
        out.push_str("## ");
        out.push_str(&block.name);
        out.push('\n');
        for (i, step) in block.steps.iter().enumerate() {
            out.push_str(&format!("{}. {}\n", i + 1, step.text));
        }
    }

    out
}
