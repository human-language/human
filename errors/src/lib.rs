mod render;

#[cfg(test)]
mod tests;

pub struct Diagnostic {
    pub file: String,
    pub line: u32,
    pub col: u16,
    pub len: u16,
    pub message: String,
}

pub fn render(d: &Diagnostic, src: Option<&[u8]>) -> String {
    let gw = render::gutter_width(&[d.line]);
    render::render_one(d, src, gw)
}

pub fn render_batch(ds: &[Diagnostic], src: Option<&[u8]>) -> String {
    if ds.is_empty() {
        return String::new();
    }
    let lines: Vec<u32> = ds.iter().map(|d| d.line).collect();
    let gw = render::gutter_width(&lines);
    let mut out = String::new();
    for (i, d) in ds.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&render::render_one(d, src, gw));
    }
    out
}
