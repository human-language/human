use std::path::Path;
use std::process;

use human_compiler::OutputFormat;
use human_errors::Diagnostic;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        help();
    }
    match args[1].as_str() {
        "validate" => cmd_validate(&args[2..]),
        "compile" => cmd_compile(&args[2..]),
        "fmt" => cmd_fmt(&args[2..]),
        "--help" | "-h" => help(),
        _ => die(&format!("unknown command: {}", args[1])),
    }
}

fn die(msg: &str) -> ! {
    eprintln!("hmn: {msg}");
    process::exit(2);
}

fn help() -> ! {
    eprint!("\
usage: hmn <command> [flags] file...

commands:
  validate   check .hmn files for errors
  compile    compile .hmn to prompt, json, yaml, toml, txt, hmn
  fmt        normalize .hmn source formatting
");
    process::exit(0);
}

fn check_help(args: &[String], usage: &str) {
    if let Some(first) = args.first() {
        if first == "--help" || first == "-h" {
            eprintln!("{usage}");
            process::exit(0);
        }
    }
}

// ── Pipeline helpers ──

fn resolve_file(path: &Path) -> Result<(human_resolver::Resolved, std::path::PathBuf), ()> {
    let canonical = std::fs::canonicalize(path).map_err(|e| {
        let d = Diagnostic {
            file: path.display().to_string(),
            line: 0, col: 0, len: 0,
            message: e.to_string(),
        };
        eprint!("{}", human_errors::render(&d, None));
    })?;
    let project_root = canonical.parent().unwrap_or(Path::new("."));
    let resolved = human_resolver::resolve(&canonical, project_root).map_err(|errors| {
        for (i, e) in errors.iter().enumerate() {
            if i > 0 {
                eprint!("\n");
            }
            let d = Diagnostic {
                file: e.file.display().to_string(),
                line: e.line.unwrap_or(0),
                col: e.col.unwrap_or(0),
                len: 0,
                message: e.message.clone(),
            };
            let src = if e.line.is_some() {
                std::fs::read(&e.file).ok()
            } else {
                None
            };
            eprint!("{}", human_errors::render(&d, src.as_deref()));
        }
    })?;
    Ok((resolved, canonical))
}

fn read_and_parse(path: &Path) -> Result<human_parser::HmnFile, ()> {
    let bytes = std::fs::read(path).map_err(|e| {
        let d = Diagnostic {
            file: path.display().to_string(),
            line: 0, col: 0, len: 0,
            message: e.to_string(),
        };
        eprint!("{}", human_errors::render(&d, None));
    })?;
    let filename = path.display().to_string();
    let tokens = human_lexer::Lexer::new(&bytes).tokenize().map_err(|errors| {
        let ds: Vec<_> = errors.iter().map(|e| Diagnostic {
            file: filename.clone(),
            line: e.line,
            col: e.col,
            len: 0,
            message: e.message.clone(),
        }).collect();
        eprint!("{}", human_errors::render_batch(&ds, Some(&bytes)));
    })?;
    let hmn_file = human_parser::parse(&tokens).map_err(|errors| {
        let ds: Vec<_> = errors.iter().map(|e| Diagnostic {
            file: filename.clone(),
            line: e.span.line,
            col: e.span.col,
            len: e.span.len,
            message: e.message.clone(),
        }).collect();
        eprint!("{}", human_errors::render_batch(&ds, Some(&bytes)));
    })?;
    Ok(hmn_file)
}

// ── Commands ──

fn cmd_validate(args: &[String]) {
    check_help(args, "usage: hmn validate file...");
    if args.is_empty() {
        die("usage: hmn validate file...");
    }
    let mut failed = false;
    for arg in args {
        if resolve_file(Path::new(arg)).is_err() {
            failed = true;
        }
    }
    process::exit(if failed { 1 } else { 0 });
}

fn cmd_compile(args: &[String]) {
    check_help(
        args,
        "usage: hmn compile [-f format] file\n  formats: prompt (default), json, yaml, toml, txt, hmn",
    );
    let (format, file) = parse_compile_args(args);
    let path = Path::new(&file);
    let (resolved, canonical) = resolve_file(path).unwrap_or_else(|()| process::exit(1));
    match human_compiler::compile(&resolved, &canonical, format) {
        Ok(output) => print!("{output}"),
        Err(e) => {
            let d = Diagnostic {
                file: e.file.display().to_string(),
                line: 0, col: 0, len: 0,
                message: e.message.clone(),
            };
            eprint!("{}", human_errors::render(&d, None));
            process::exit(1);
        }
    }
}

fn cmd_fmt(args: &[String]) {
    check_help(args, "usage: hmn fmt [-w] file...");
    let (write_back, files) = parse_fmt_args(args);
    if files.is_empty() {
        die("usage: hmn fmt [-w] file...");
    }
    let mut failed = false;
    for file in &files {
        let path = Path::new(file);
        match read_and_parse(path) {
            Ok(hmn_file) => {
                let output = human_compiler::hmn::emit_file(&hmn_file);
                if write_back {
                    if let Err(e) = std::fs::write(path, &output) {
                        eprintln!("hmn: {}: {e}", path.display());
                        failed = true;
                    }
                } else {
                    print!("{output}");
                }
            }
            Err(()) => {
                failed = true;
            }
        }
    }
    process::exit(if failed { 1 } else { 0 });
}

// ── Arg parsing helpers ──

fn parse_format(s: &str) -> OutputFormat {
    match s {
        "prompt" => OutputFormat::Prompt,
        "json" => OutputFormat::Json,
        "yaml" => OutputFormat::Yaml,
        "toml" => OutputFormat::Toml,
        "txt" => OutputFormat::Txt,
        "hmn" => OutputFormat::Hmn,
        _ => die(&format!("unknown format: {s}")),
    }
}

fn parse_compile_args(args: &[String]) -> (OutputFormat, String) {
    match args.len() {
        1 if !args[0].starts_with('-') => (OutputFormat::Prompt, args[0].clone()),
        3 if args[0] == "-f" => (parse_format(&args[1]), args[2].clone()),
        _ => die("usage: hmn compile [-f format] file"),
    }
}

fn parse_fmt_args(args: &[String]) -> (bool, Vec<String>) {
    if args.is_empty() {
        return (false, vec![]);
    }
    if args[0] == "-w" {
        (true, args[1..].to_vec())
    } else {
        (false, args.to_vec())
    }
}
