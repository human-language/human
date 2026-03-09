use crate::Diagnostic;

pub fn extract_line(src: &[u8], line: u32) -> Option<&[u8]> {
    if line == 0 || src.is_empty() {
        return None;
    }
    let target = line as usize;
    let mut current = 1usize;
    let mut start = 0usize;
    let mut i = 0;

    while i < src.len() {
        if src[i] == b'\n' || src[i] == b'\r' {
            if current == target {
                return Some(&src[start..i]);
            }
            // skip \r\n as one newline
            if src[i] == b'\r' && i + 1 < src.len() && src[i + 1] == b'\n' {
                i += 1;
            }
            current += 1;
            start = i + 1;
        }
        i += 1;
    }

    if current == target && start <= src.len() {
        let slice = &src[start..];
        if slice.is_empty() {
            return None;
        }
        return Some(slice);
    }

    None
}

pub fn gutter_width(lines: &[u32]) -> usize {
    let max_line = lines.iter().copied().filter(|&l| l > 0).max().unwrap_or(1);
    let mut w = 0usize;
    let mut n = max_line;
    while n > 0 {
        w += 1;
        n /= 10;
    }
    if w == 0 { 1 } else { w }
}

fn make_carets(len: u16, max_len: usize) -> String {
    if max_len == 0 {
        return String::new();
    }
    if len <= 1 {
        return "^".to_string();
    }
    let total = (len as usize).min(max_len);
    let mut s = String::with_capacity(total);
    s.push('^');
    for _ in 1..total {
        s.push('~');
    }
    s
}

pub fn render_one(d: &Diagnostic, src: Option<&[u8]>, gw: usize) -> String {
    let mut out = String::new();

    // Line 1: error message
    out.push_str("error: ");
    out.push_str(&d.message);
    out.push('\n');

    // Line 2: location
    if d.line == 0 && d.col == 0 {
        out.push_str(" --> ");
        out.push_str(&d.file);
        out.push('\n');
        return out;
    }

    out.push_str(" --> ");
    out.push_str(&d.file);
    out.push(':');
    out.push_str(&d.line.to_string());
    out.push(':');
    out.push_str(&d.col.to_string());
    out.push('\n');

    let src = match src {
        Some(s) => s,
        None => return out,
    };

    let line_bytes = match extract_line(src, d.line) {
        Some(b) => b,
        None => return out,
    };

    let line_text = std::str::from_utf8(line_bytes).unwrap_or("<non-utf8>");

    // Blank gutter
    for _ in 0..gw {
        out.push(' ');
    }
    out.push_str(" |\n");

    // Source line
    let line_str = d.line.to_string();
    for _ in 0..(gw - line_str.len()) {
        out.push(' ');
    }
    out.push_str(&line_str);
    out.push_str(" | ");
    out.push_str(line_text);
    out.push('\n');

    // Caret row
    let col_idx = d.col as usize;
    if col_idx == 0 || col_idx > line_text.len() {
        return out;
    }
    let remaining = line_text.len() - (col_idx - 1);
    let carets = make_carets(d.len, remaining);
    if carets.is_empty() {
        return out;
    }

    for _ in 0..gw {
        out.push(' ');
    }
    out.push_str(" | ");
    for _ in 0..(col_idx - 1) {
        out.push(' ');
    }
    out.push_str(&carets);
    out.push('\n');

    out
}
