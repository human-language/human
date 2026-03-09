use std::path::PathBuf;
use tempfile::TempDir;
use human_lexer::Span;
use human_parser::{
    AgentDecl, ConstraintsBlock, Constraint, ConstraintLevel,
    FlowBlock, FlowStep, HmnFile, Import, ImportTarget, Property,
    SystemDecl, TestBlock, TestExpect, TestInput, TestOp, Value,
};
use human_resolver::Resolved;
use crate::{compile, compile_hmn, hmn::emit_file, OutputFormat};

fn span() -> Span {
    Span { offset: 0, len: 0, line: 1, col: 1 }
}

fn make_agent(name: &str, system: Option<&str>, props: Vec<Property>) -> AgentDecl {
    AgentDecl {
        name: name.to_string(),
        properties: props,
        system: system.map(|p| SystemDecl {
            path: p.to_string(),
            span: span(),
        }),
        span: span(),
    }
}

fn make_constraint(level: ConstraintLevel, text: &str) -> Constraint {
    Constraint {
        level,
        text: text.to_string(),
        span: span(),
    }
}

fn make_constraints_block(name: &str, rules: Vec<Constraint>) -> ConstraintsBlock {
    ConstraintsBlock {
        name: name.to_string(),
        constraints: rules,
        span: span(),
    }
}

fn make_flow(name: &str, steps: Vec<&str>) -> FlowBlock {
    FlowBlock {
        name: name.to_string(),
        steps: steps.into_iter().map(|s| FlowStep {
            text: s.to_string(),
            span: span(),
        }).collect(),
        span: span(),
    }
}

fn make_prop(key: &str, val: Value) -> Property {
    Property {
        key: key.to_string(),
        value: val,
        span: span(),
    }
}

fn dennis_resolved(system_path: Option<&str>) -> Resolved {
    Resolved {
        agent: make_agent("dennis", system_path, vec![]),
        constraints: vec![
            make_constraints_block("mentorship_discipline", vec![
                make_constraint(ConstraintLevel::Never, "skip phases"),
                make_constraint(ConstraintLevel::Never, "accept vague problem statements"),
                make_constraint(ConstraintLevel::Must, "follow phase order strictly"),
                make_constraint(ConstraintLevel::Must, "critique existing work before proceeding"),
                make_constraint(ConstraintLevel::Should, "use socratic method"),
                make_constraint(ConstraintLevel::Avoid, "over explaining"),
                make_constraint(ConstraintLevel::May, "push back hard on weak reasoning"),
            ]),
            make_constraints_block("interaction_quality", vec![
                make_constraint(ConstraintLevel::Never, "produce caricature of ritchie"),
                make_constraint(ConstraintLevel::Must, "be terse and direct"),
                make_constraint(ConstraintLevel::Should, "back claims with precedent"),
            ]),
        ],
        flows: vec![
            make_flow("mentorship_session", vec![
                "assess existing work",
                "identify current phase",
                "critique and question",
                "guide to deliverable",
                "lock decisions",
            ]),
        ],
        tests: vec![],
        sources: vec![PathBuf::from("main.hmn")],
    }
}

fn write_system_file(dir: &TempDir, name: &str, content: &str) {
    std::fs::write(dir.path().join(name), content).unwrap();
}

// ── HMN tests ──

#[test]
fn hmn_full_output() {
    let resolved = dennis_resolved(Some("./dennis.md"));
    let out = compile_hmn(&resolved);
    assert!(out.contains("AGENT dennis\n"));
    assert!(out.contains("SYSTEM ./dennis.md\n"));
    assert!(out.contains("CONSTRAINTS mentorship_discipline\n"));
    assert!(out.contains("  NEVER skip phases\n"));
    assert!(out.contains("  MUST follow phase order strictly\n"));
    assert!(out.contains("  SHOULD use socratic method\n"));
    assert!(out.contains("  AVOID over explaining\n"));
    assert!(out.contains("  MAY push back hard on weak reasoning\n"));
    assert!(out.contains("CONSTRAINTS interaction_quality\n"));
    assert!(out.contains("FLOW mentorship_session\n"));
    assert!(out.contains("  assess existing work\n"));
    assert!(out.contains("  lock decisions\n"));
    assert!(!out.contains("IMPORT"));
}

#[test]
fn hmn_no_system() {
    let resolved = dennis_resolved(None);
    let out = compile_hmn(&resolved);
    assert!(out.contains("AGENT dennis\n"));
    assert!(!out.contains("SYSTEM"));
}

#[test]
fn hmn_properties() {
    let resolved = Resolved {
        agent: make_agent("bot", None, vec![
            make_prop("model", Value::Str("gpt-4".to_string())),
            make_prop("verbose", Value::Bool(true)),
            make_prop("temperature", Value::Number(0.7)),
        ]),
        constraints: vec![],
        flows: vec![],
        tests: vec![],
        sources: vec![PathBuf::from("main.hmn")],
    };
    let out = compile_hmn(&resolved);
    assert!(out.contains("AGENT bot\n"));
    assert!(out.contains("  model = \"gpt-4\"\n"));
    assert!(out.contains("  verbose = true\n"));
    assert!(out.contains("  temperature = 0.7\n"));
}

#[test]
fn hmn_fixed_point() {
    let resolved = dennis_resolved(Some("./dennis.md"));
    let first = compile_hmn(&resolved);
    let tokens = human_lexer::Lexer::new(first.as_bytes()).tokenize().unwrap();
    let parsed = human_parser::parse(&tokens).unwrap();
    let second_resolved = Resolved {
        agent: parsed.agent.unwrap(),
        constraints: parsed.constraints,
        flows: parsed.flows,
        tests: vec![],
        sources: vec![PathBuf::from("main.hmn")],
    };
    let second = compile_hmn(&second_resolved);
    assert_eq!(first, second, "HMN output is not a fixed point");
}

#[test]
fn hmn_multiple_blocks() {
    let resolved = Resolved {
        agent: make_agent("bot", None, vec![]),
        constraints: vec![
            make_constraints_block("safety", vec![
                make_constraint(ConstraintLevel::Never, "lie"),
            ]),
            make_constraints_block("tone", vec![
                make_constraint(ConstraintLevel::Must, "be polite"),
            ]),
        ],
        flows: vec![
            make_flow("greet", vec!["say hello"]),
            make_flow("farewell", vec!["say goodbye"]),
        ],
        tests: vec![],
        sources: vec![PathBuf::from("main.hmn")],
    };
    let out = compile_hmn(&resolved);
    let safety_pos = out.find("CONSTRAINTS safety").unwrap();
    let tone_pos = out.find("CONSTRAINTS tone").unwrap();
    let greet_pos = out.find("FLOW greet").unwrap();
    let farewell_pos = out.find("FLOW farewell").unwrap();
    assert!(safety_pos < tone_pos);
    assert!(tone_pos < greet_pos);
    assert!(greet_pos < farewell_pos);
}

// ── Prompt tests ──

#[test]
fn prompt_with_system() {
    let tmp = TempDir::new().unwrap();
    write_system_file(&tmp, "dennis.md", "You are Dennis Ritchie. You mentor through design.\n");
    let resolved = dennis_resolved(Some("./dennis.md"));
    let root = tmp.path().join("main.hmn");
    let out = compile(&resolved, &root, OutputFormat::Prompt).unwrap();
    assert!(out.starts_with("You are Dennis Ritchie."));
    assert!(out.contains("---"));
    assert!(out.contains("# dennis\n"));
    assert!(out.contains("## mentorship_discipline\n"));
    assert!(out.contains("- NEVER: skip phases\n"));
    assert!(out.contains("## mentorship_session\n"));
    assert!(out.contains("1. assess existing work\n"));
}

#[test]
fn prompt_without_system() {
    let resolved = dennis_resolved(None);
    let root = PathBuf::from("/tmp/main.hmn");
    let out = compile(&resolved, &root, OutputFormat::Prompt).unwrap();
    assert!(out.starts_with("# dennis\n"));
    assert!(!out.contains("---"));
}

#[test]
fn prompt_constraint_levels() {
    let tmp = TempDir::new().unwrap();
    let resolved = Resolved {
        agent: make_agent("bot", None, vec![]),
        constraints: vec![make_constraints_block("all_levels", vec![
            make_constraint(ConstraintLevel::Never, "a"),
            make_constraint(ConstraintLevel::Must, "b"),
            make_constraint(ConstraintLevel::Should, "c"),
            make_constraint(ConstraintLevel::Avoid, "d"),
            make_constraint(ConstraintLevel::May, "e"),
        ])],
        flows: vec![],
        tests: vec![],
        sources: vec![tmp.path().join("main.hmn")],
    };
    let root = tmp.path().join("main.hmn");
    let out = compile(&resolved, &root, OutputFormat::Prompt).unwrap();
    assert!(out.contains("- NEVER: a\n"));
    assert!(out.contains("- MUST: b\n"));
    assert!(out.contains("- SHOULD: c\n"));
    assert!(out.contains("- AVOID: d\n"));
    assert!(out.contains("- MAY: e\n"));
}

#[test]
fn prompt_multiple_blocks() {
    let tmp = TempDir::new().unwrap();
    let resolved = Resolved {
        agent: make_agent("bot", None, vec![]),
        constraints: vec![
            make_constraints_block("safety", vec![
                make_constraint(ConstraintLevel::Never, "lie"),
            ]),
            make_constraints_block("tone", vec![
                make_constraint(ConstraintLevel::Must, "be polite"),
            ]),
        ],
        flows: vec![
            make_flow("greet", vec!["say hello"]),
            make_flow("farewell", vec!["say goodbye"]),
        ],
        tests: vec![],
        sources: vec![tmp.path().join("main.hmn")],
    };
    let root = tmp.path().join("main.hmn");
    let out = compile(&resolved, &root, OutputFormat::Prompt).unwrap();
    assert!(out.contains("## safety\n"));
    assert!(out.contains("## tone\n"));
    assert!(out.contains("## greet\n"));
    assert!(out.contains("## farewell\n"));
}

#[test]
fn prompt_agent_properties() {
    let tmp = TempDir::new().unwrap();
    let resolved = Resolved {
        agent: make_agent("bot", None, vec![
            make_prop("model", Value::Str("gpt-4".to_string())),
            make_prop("verbose", Value::Bool(true)),
        ]),
        constraints: vec![],
        flows: vec![],
        tests: vec![],
        sources: vec![tmp.path().join("main.hmn")],
    };
    let root = tmp.path().join("main.hmn");
    let out = compile(&resolved, &root, OutputFormat::Prompt).unwrap();
    assert!(out.contains("model = gpt-4\n"));
    assert!(out.contains("verbose = true\n"));
}

// ── JSON tests ──

#[test]
fn json_full_output() {
    let tmp = TempDir::new().unwrap();
    write_system_file(&tmp, "dennis.md", "You are Dennis Ritchie.\n");
    let resolved = dennis_resolved(Some("./dennis.md"));
    let root = tmp.path().join("main.hmn");
    let out = compile(&resolved, &root, OutputFormat::Json).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["name"], "dennis");
    assert!(v["system"].as_str().unwrap().contains("Dennis Ritchie"));
    assert!(v["constraints"]["mentorship_discipline"].is_array());
    assert!(v["flows"]["mentorship_session"].is_array());
}

#[test]
fn json_properties() {
    let tmp = TempDir::new().unwrap();
    let resolved = Resolved {
        agent: make_agent("bot", None, vec![
            make_prop("model", Value::Str("gpt-4".to_string())),
            make_prop("verbose", Value::Bool(true)),
            make_prop("temperature", Value::Number(0.7)),
            make_prop("config", Value::Path("./config.toml".to_string())),
        ]),
        constraints: vec![],
        flows: vec![],
        tests: vec![],
        sources: vec![tmp.path().join("main.hmn")],
    };
    let root = tmp.path().join("main.hmn");
    let out = compile(&resolved, &root, OutputFormat::Json).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["properties"]["model"], "gpt-4");
    assert_eq!(v["properties"]["verbose"], true);
    assert_eq!(v["properties"]["temperature"], 0.7);
    assert_eq!(v["properties"]["config"], "./config.toml");
}

#[test]
fn json_no_system() {
    let tmp = TempDir::new().unwrap();
    let resolved = Resolved {
        agent: make_agent("bot", None, vec![]),
        constraints: vec![],
        flows: vec![],
        tests: vec![],
        sources: vec![tmp.path().join("main.hmn")],
    };
    let root = tmp.path().join("main.hmn");
    let out = compile(&resolved, &root, OutputFormat::Json).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(v.get("system").is_none());
}

#[test]
fn json_constraints_format() {
    let tmp = TempDir::new().unwrap();
    let resolved = Resolved {
        agent: make_agent("bot", None, vec![]),
        constraints: vec![make_constraints_block("safety", vec![
            make_constraint(ConstraintLevel::Never, "lie"),
            make_constraint(ConstraintLevel::Must, "be honest"),
        ])],
        flows: vec![],
        tests: vec![],
        sources: vec![tmp.path().join("main.hmn")],
    };
    let root = tmp.path().join("main.hmn");
    let out = compile(&resolved, &root, OutputFormat::Json).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let rules = v["constraints"]["safety"].as_array().unwrap();
    assert_eq!(rules[0], "NEVER lie");
    assert_eq!(rules[1], "MUST be honest");
}

// ── YAML tests ──

#[test]
fn yaml_full_output() {
    let tmp = TempDir::new().unwrap();
    write_system_file(&tmp, "dennis.md", "You are Dennis Ritchie.\n");
    let resolved = dennis_resolved(Some("./dennis.md"));
    let root = tmp.path().join("main.hmn");
    let out = compile(&resolved, &root, OutputFormat::Yaml).unwrap();
    let v: serde_yaml::Value = serde_yaml::from_str(&out).unwrap();
    assert_eq!(v["name"].as_str().unwrap(), "dennis");
    assert!(v["system"].as_str().unwrap().contains("Dennis Ritchie"));
    let md = &v["constraints"]["mentorship_discipline"];
    assert!(md.is_sequence(), "mentorship_discipline should be a sequence");
    let iq = &v["constraints"]["interaction_quality"];
    assert!(iq.is_sequence(), "interaction_quality should be a sequence");
    let ms = &v["flows"]["mentorship_session"];
    assert!(ms.is_sequence(), "mentorship_session should be a sequence");
}

#[test]
fn yaml_no_system() {
    let tmp = TempDir::new().unwrap();
    let resolved = Resolved {
        agent: make_agent("bot", None, vec![]),
        constraints: vec![],
        flows: vec![],
        tests: vec![],
        sources: vec![tmp.path().join("main.hmn")],
    };
    let root = tmp.path().join("main.hmn");
    let out = compile(&resolved, &root, OutputFormat::Yaml).unwrap();
    let v: serde_yaml::Value = serde_yaml::from_str(&out).unwrap();
    assert!(v.get("system").is_none(), "system key should be absent");
}

#[test]
fn yaml_multiline_system() {
    let tmp = TempDir::new().unwrap();
    write_system_file(&tmp, "sys.md", "Line one.\n\nLine two.\n\nLine three.\n");
    let resolved = Resolved {
        agent: make_agent("bot", Some("./sys.md"), vec![]),
        constraints: vec![],
        flows: vec![],
        tests: vec![],
        sources: vec![tmp.path().join("main.hmn")],
    };
    let root = tmp.path().join("main.hmn");
    let out = compile(&resolved, &root, OutputFormat::Yaml).unwrap();
    assert!(out.contains("system:") || out.contains("system: "));
    let v: serde_yaml::Value = serde_yaml::from_str(&out).unwrap();
    let sys = v["system"].as_str().unwrap();
    assert!(sys.contains("Line one."));
    assert!(sys.contains("Line three."));
}

// ── TOML tests ──

#[test]
fn toml_full_output() {
    let tmp = TempDir::new().unwrap();
    write_system_file(&tmp, "dennis.md", "You are Dennis Ritchie.\n");
    let resolved = dennis_resolved(Some("./dennis.md"));
    let root = tmp.path().join("main.hmn");
    let out = compile(&resolved, &root, OutputFormat::Toml).unwrap();
    assert!(out.contains("name = \"dennis\""));
    assert!(out.contains("[constraints]"));
    assert!(out.contains("[flows]"));
}

#[test]
fn toml_properties() {
    let tmp = TempDir::new().unwrap();
    let resolved = Resolved {
        agent: make_agent("bot", None, vec![
            make_prop("model", Value::Str("gpt-4".to_string())),
            make_prop("verbose", Value::Bool(true)),
            make_prop("temperature", Value::Number(0.7)),
        ]),
        constraints: vec![],
        flows: vec![],
        tests: vec![],
        sources: vec![tmp.path().join("main.hmn")],
    };
    let root = tmp.path().join("main.hmn");
    let out = compile(&resolved, &root, OutputFormat::Toml).unwrap();
    assert!(out.contains("model = \"gpt-4\""));
    assert!(out.contains("verbose = true"));
    assert!(out.contains("temperature = 0.7"));
}

#[test]
fn toml_no_system() {
    let tmp = TempDir::new().unwrap();
    let resolved = Resolved {
        agent: make_agent("bot", None, vec![]),
        constraints: vec![],
        flows: vec![],
        tests: vec![],
        sources: vec![tmp.path().join("main.hmn")],
    };
    let root = tmp.path().join("main.hmn");
    let out = compile(&resolved, &root, OutputFormat::Toml).unwrap();
    let v: toml::Value = toml::from_str(&out).unwrap();
    assert!(v.get("system").is_none(), "system key should be absent");
}

// ── TXT tests ──

#[test]
fn txt_full_output() {
    let tmp = TempDir::new().unwrap();
    write_system_file(&tmp, "dennis.md", "You are Dennis Ritchie.\n");
    let resolved = dennis_resolved(Some("./dennis.md"));
    let root = tmp.path().join("main.hmn");
    let out = compile(&resolved, &root, OutputFormat::Txt).unwrap();
    assert!(out.contains("AGENT dennis\n"));
    assert!(out.contains("SYSTEM You are Dennis Ritchie.\n"));
    assert!(out.contains("CONSTRAINTS mentorship_discipline\n"));
    assert!(out.contains("  NEVER skip phases\n"));
    assert!(out.contains("FLOW mentorship_session\n"));
    assert!(out.contains("  assess existing work\n"));
}

#[test]
fn txt_no_system() {
    let tmp = TempDir::new().unwrap();
    let resolved = dennis_resolved(None);
    let root = tmp.path().join("main.hmn");
    let out = compile(&resolved, &root, OutputFormat::Txt).unwrap();
    assert!(out.contains("AGENT dennis\n"));
    assert!(!out.contains("SYSTEM"));
}

#[test]
fn txt_idempotent() {
    let tmp = TempDir::new().unwrap();
    let resolved = Resolved {
        agent: make_agent("bot", None, vec![]),
        constraints: vec![make_constraints_block("safety", vec![
            make_constraint(ConstraintLevel::Never, "lie"),
        ])],
        flows: vec![make_flow("greet", vec!["say hello"])],
        tests: vec![],
        sources: vec![tmp.path().join("main.hmn")],
    };
    let root = tmp.path().join("main.hmn");
    let out = compile(&resolved, &root, OutputFormat::Txt).unwrap();
    assert!(out.contains("AGENT bot\n"));
    assert!(out.contains("CONSTRAINTS safety\n"));
    assert!(out.contains("  NEVER lie\n"));
    assert!(out.contains("FLOW greet\n"));
    assert!(out.contains("  say hello\n"));
}

// ── OutputFormat dispatch ──

#[test]
fn compile_dispatches() {
    let tmp = TempDir::new().unwrap();
    write_system_file(&tmp, "sys.md", "system content\n");
    let resolved = Resolved {
        agent: make_agent("bot", Some("./sys.md"), vec![]),
        constraints: vec![make_constraints_block("safety", vec![
            make_constraint(ConstraintLevel::Never, "lie"),
        ])],
        flows: vec![make_flow("greet", vec!["hello"])],
        tests: vec![],
        sources: vec![tmp.path().join("main.hmn")],
    };
    let root = tmp.path().join("main.hmn");

    let hmn = compile(&resolved, &root, OutputFormat::Hmn).unwrap();
    assert!(hmn.contains("AGENT bot"));
    assert!(hmn.contains("SYSTEM ./sys.md"));

    let json = compile(&resolved, &root, OutputFormat::Json).unwrap();
    assert!(json.starts_with('{'));

    let yaml = compile(&resolved, &root, OutputFormat::Yaml).unwrap();
    assert!(yaml.contains("name: bot"));

    let toml_out = compile(&resolved, &root, OutputFormat::Toml).unwrap();
    assert!(toml_out.contains("[constraints]"));

    let prompt = compile(&resolved, &root, OutputFormat::Prompt).unwrap();
    assert!(prompt.contains("# bot"));

    let txt = compile(&resolved, &root, OutputFormat::Txt).unwrap();
    assert!(txt.contains("AGENT bot"));
    assert!(txt.contains("SYSTEM system content"));
}

// ── Error tests ──

#[test]
fn system_file_not_found() {
    let tmp = TempDir::new().unwrap();
    let resolved = Resolved {
        agent: make_agent("bot", Some("./nonexistent.md"), vec![]),
        constraints: vec![],
        flows: vec![],
        tests: vec![],
        sources: vec![tmp.path().join("main.hmn")],
    };
    let root = tmp.path().join("main.hmn");
    let err = compile(&resolved, &root, OutputFormat::Prompt).unwrap_err();
    assert!(err.message.contains("file not found"), "got: {}", err);
}

#[test]
fn system_file_not_utf8() {
    let tmp = TempDir::new().unwrap();
    let bad_bytes: &[u8] = &[0xFF, 0xFE, 0x80, 0x81];
    std::fs::write(tmp.path().join("bad.md"), bad_bytes).unwrap();
    let resolved = Resolved {
        agent: make_agent("bot", Some("./bad.md"), vec![]),
        constraints: vec![],
        flows: vec![],
        tests: vec![],
        sources: vec![tmp.path().join("main.hmn")],
    };
    let root = tmp.path().join("main.hmn");
    let err = compile(&resolved, &root, OutputFormat::Prompt).unwrap_err();
    assert!(
        err.message.contains("UTF-8") || err.message.contains("cannot read"),
        "got: {}", err
    );
}

// ── Edge case tests ──

#[test]
fn empty_constraints() {
    let tmp = TempDir::new().unwrap();
    let resolved = Resolved {
        agent: make_agent("bot", None, vec![]),
        constraints: vec![],
        flows: vec![make_flow("greet", vec!["hello"])],
        tests: vec![],
        sources: vec![tmp.path().join("main.hmn")],
    };
    let root = tmp.path().join("main.hmn");

    let prompt = compile(&resolved, &root, OutputFormat::Prompt).unwrap();
    assert!(!prompt.contains("NEVER"));
    assert!(!prompt.contains("MUST"));

    let json = compile(&resolved, &root, OutputFormat::Json).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("constraints").is_none());

    let hmn = compile_hmn(&resolved);
    assert!(!hmn.contains("CONSTRAINTS"));
}

#[test]
fn empty_flows() {
    let tmp = TempDir::new().unwrap();
    let resolved = Resolved {
        agent: make_agent("bot", None, vec![]),
        constraints: vec![make_constraints_block("safety", vec![
            make_constraint(ConstraintLevel::Never, "lie"),
        ])],
        flows: vec![],
        tests: vec![],
        sources: vec![tmp.path().join("main.hmn")],
    };
    let root = tmp.path().join("main.hmn");

    let prompt = compile(&resolved, &root, OutputFormat::Prompt).unwrap();
    assert!(!prompt.contains("FLOW"));

    let json = compile(&resolved, &root, OutputFormat::Json).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("flows").is_none());

    let hmn = compile_hmn(&resolved);
    assert!(!hmn.contains("FLOW"));
}

#[test]
fn system_file_empty() {
    let tmp = TempDir::new().unwrap();
    write_system_file(&tmp, "empty.md", "");
    let resolved = Resolved {
        agent: make_agent("bot", Some("./empty.md"), vec![]),
        constraints: vec![],
        flows: vec![],
        tests: vec![],
        sources: vec![tmp.path().join("main.hmn")],
    };
    let root = tmp.path().join("main.hmn");

    let prompt = compile(&resolved, &root, OutputFormat::Prompt).unwrap();
    assert!(prompt.starts_with("# bot\n"), "prompt should start with # bot, got: {}", prompt);
    assert!(!prompt.contains("---"));

    let hmn = compile_hmn(&resolved);
    assert!(hmn.contains("SYSTEM ./empty.md"));
}

#[test]
fn constraint_block_no_rules() {
    let tmp = TempDir::new().unwrap();
    let resolved = Resolved {
        agent: make_agent("bot", None, vec![]),
        constraints: vec![make_constraints_block("empty_block", vec![])],
        flows: vec![],
        tests: vec![],
        sources: vec![tmp.path().join("main.hmn")],
    };
    let root = tmp.path().join("main.hmn");

    let prompt = compile(&resolved, &root, OutputFormat::Prompt).unwrap();
    assert!(prompt.contains("## empty_block\n"));

    let hmn = compile_hmn(&resolved);
    assert!(hmn.contains("CONSTRAINTS empty_block\n"));
}

#[test]
fn hmn_no_fail() {
    let resolved = Resolved {
        agent: make_agent("minimal", None, vec![]),
        constraints: vec![],
        flows: vec![],
        tests: vec![],
        sources: vec![PathBuf::from("main.hmn")],
    };
    let out = compile_hmn(&resolved);
    assert_eq!(out, "AGENT minimal\n");
}

// ── New tests: string escaping, properties, multi-line, multiple flows ──

#[test]
fn hmn_string_escaping() {
    let resolved = Resolved {
        agent: make_agent("bot", None, vec![
            make_prop("greeting", Value::Str("say \"hello\" to\\from".to_string())),
        ]),
        constraints: vec![],
        flows: vec![],
        tests: vec![],
        sources: vec![PathBuf::from("main.hmn")],
    };
    let first = compile_hmn(&resolved);
    assert!(first.contains(r#"  greeting = "say \"hello\" to\\from""#),
        "escaped output wrong: {}", first);

    let tokens = human_lexer::Lexer::new(first.as_bytes()).tokenize().unwrap();
    let parsed = human_parser::parse(&tokens).unwrap();
    let agent = parsed.agent.unwrap();
    assert_eq!(agent.properties[0].key, "greeting");
    assert_eq!(
        agent.properties[0].value,
        Value::Str("say \"hello\" to\\from".to_string()),
        "round-trip failed"
    );
}

#[test]
fn yaml_properties() {
    let tmp = TempDir::new().unwrap();
    let resolved = Resolved {
        agent: make_agent("bot", None, vec![
            make_prop("model", Value::Str("gpt-4".to_string())),
            make_prop("verbose", Value::Bool(true)),
            make_prop("temperature", Value::Number(0.7)),
        ]),
        constraints: vec![],
        flows: vec![],
        tests: vec![],
        sources: vec![tmp.path().join("main.hmn")],
    };
    let root = tmp.path().join("main.hmn");
    let out = compile(&resolved, &root, OutputFormat::Yaml).unwrap();
    let v: serde_yaml::Value = serde_yaml::from_str(&out).unwrap();
    assert_eq!(v["properties"]["model"].as_str().unwrap(), "gpt-4");
    assert_eq!(v["properties"]["verbose"].as_bool().unwrap(), true);
    let temp = v["properties"]["temperature"].as_f64().unwrap();
    assert!((temp - 0.7).abs() < f64::EPSILON);
}

#[test]
fn txt_properties() {
    let tmp = TempDir::new().unwrap();
    let resolved = Resolved {
        agent: make_agent("bot", None, vec![
            make_prop("model", Value::Str("gpt-4".to_string())),
            make_prop("verbose", Value::Bool(true)),
        ]),
        constraints: vec![],
        flows: vec![],
        tests: vec![],
        sources: vec![tmp.path().join("main.hmn")],
    };
    let root = tmp.path().join("main.hmn");
    let out = compile(&resolved, &root, OutputFormat::Txt).unwrap();
    assert!(out.contains("MODEL gpt-4\n"), "got: {}", out);
    assert!(out.contains("VERBOSE true\n"), "got: {}", out);
}

#[test]
fn txt_multiline_system() {
    let tmp = TempDir::new().unwrap();
    write_system_file(&tmp, "sys.md", "First line.\n\nSecond line.\n\nThird line.\n");
    let resolved = Resolved {
        agent: make_agent("bot", Some("./sys.md"), vec![]),
        constraints: vec![],
        flows: vec![],
        tests: vec![],
        sources: vec![tmp.path().join("main.hmn")],
    };
    let root = tmp.path().join("main.hmn");
    let out = compile(&resolved, &root, OutputFormat::Txt).unwrap();
    assert!(out.contains("SYSTEM First line.\n"), "got: {}", out);
    assert!(!out.contains("Second line."), "TXT should only show first line");
    assert!(!out.contains("Third line."), "TXT should only show first line");
}

#[test]
fn json_multiple_flows() {
    let tmp = TempDir::new().unwrap();
    let resolved = Resolved {
        agent: make_agent("bot", None, vec![]),
        constraints: vec![],
        flows: vec![
            make_flow("greet", vec!["say hello", "wave"]),
            make_flow("farewell", vec!["say goodbye"]),
        ],
        tests: vec![],
        sources: vec![tmp.path().join("main.hmn")],
    };
    let root = tmp.path().join("main.hmn");
    let out = compile(&resolved, &root, OutputFormat::Json).unwrap();
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let greet = v["flows"]["greet"].as_array().unwrap();
    assert_eq!(greet.len(), 2);
    assert_eq!(greet[0], "say hello");
    assert_eq!(greet[1], "wave");
    let farewell = v["flows"]["farewell"].as_array().unwrap();
    assert_eq!(farewell.len(), 1);
    assert_eq!(farewell[0], "say goodbye");
}

// ── emit_file tests ──

#[test]
fn emit_file_with_tests() {
    let file = HmnFile {
        imports: vec![],
        agent: Some(make_agent("bot", None, vec![])),
        constraints: vec![],
        flows: vec![],
        tests: vec![TestBlock {
            inputs: vec![
                TestInput { value: "hello".to_string(), span: span() },
            ],
            expects: vec![
                TestExpect {
                    negated: false,
                    op: TestOp::Contains,
                    value: "hi".to_string(),
                    span: span(),
                },
                TestExpect {
                    negated: true,
                    op: TestOp::Matches,
                    value: "bye.*".to_string(),
                    span: span(),
                },
            ],
            span: span(),
        }],
    };
    let out = emit_file(&file);
    assert!(out.contains("AGENT bot\n"), "out: {out}");
    assert!(out.contains("TEST\n"), "out: {out}");
    assert!(out.contains("  INPUT \"hello\"\n"), "out: {out}");
    assert!(out.contains("  EXPECT CONTAINS \"hi\"\n"), "out: {out}");
    assert!(out.contains("  EXPECT NOT MATCHES \"bye.*\"\n"), "out: {out}");
}

#[test]
fn emit_file_fragment() {
    let file = HmnFile {
        imports: vec![],
        agent: None,
        constraints: vec![make_constraints_block("safety", vec![
            make_constraint(ConstraintLevel::Never, "lie"),
        ])],
        flows: vec![make_flow("greet", vec!["hello"])],
        tests: vec![],
    };
    let out = emit_file(&file);
    assert!(!out.contains("AGENT"), "out: {out}");
    assert!(out.contains("CONSTRAINTS safety\n"), "out: {out}");
    assert!(out.contains("  NEVER lie\n"), "out: {out}");
    assert!(out.contains("FLOW greet\n"), "out: {out}");
    assert!(out.contains("  hello\n"), "out: {out}");
}

#[test]
fn emit_file_with_imports() {
    let file = HmnFile {
        imports: vec![
            Import {
                target: ImportTarget::Path("./rules.hmn".to_string()),
                span: span(),
            },
            Import {
                target: ImportTarget::Package("stdlib/safety".to_string()),
                span: span(),
            },
        ],
        agent: Some(make_agent("bot", None, vec![])),
        constraints: vec![],
        flows: vec![],
        tests: vec![],
    };
    let out = emit_file(&file);
    assert!(out.starts_with("IMPORT ./rules.hmn\n"), "out: {out}");
    assert!(out.contains("IMPORT @stdlib/safety\n"), "out: {out}");
    assert!(out.contains("AGENT bot\n"), "out: {out}");
    let import_pos = out.find("IMPORT @stdlib").unwrap();
    let agent_pos = out.find("AGENT bot").unwrap();
    assert!(import_pos < agent_pos, "imports should come before agent");
}
