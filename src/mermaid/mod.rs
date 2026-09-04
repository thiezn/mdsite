//! grok-mermaid: render Mermaid diagram source as Unicode box-drawing art.
//!
//! The actual renderer lives in `mermaid.rs`, copied (Apache-2.0) from
//! <https://github.com/xai-org/grok-build>
//! (`crates/codegen/xai-grok-markdown/src/mermaid.rs`) — see LICENSE and
//! README.md in this directory. This file adds:
//!
//! - `render_plain` / `render_html`: safe Rust entry points. The HTML flavor
//!   wraps each styled run in `<span class="...">` using the class names
//!   `b` (border), `n` (node text), `e` (edge), `el` (edge label),
//!   `t` (title), plus `i` for italic, so a page can color the art the way
//!   the original TUI does.

mod renderer;
mod shim;

use renderer::MermaidStyles;
use shim::Style;

fn styles() -> MermaidStyles {
    MermaidStyles {
        border: Style::class("b"),
        node_text: Style::class("n"),
        edge: Style::class("e"),
        edge_label: Style::class("el"),
        title: Style::class("t"),
    }
}

/// Render mermaid source to plain (uncolored) box-drawing text, or `None`
/// for blank input. `max_width` bounds the diagram width in display columns;
/// diagrams that cannot fit fall back to the framed raw source.
#[allow(dead_code)]
pub fn render_plain(src: &str, max_width: Option<usize>) -> Option<String> {
    renderer::render(src, &styles(), max_width).map(|art| art.plain_lines.join("\n"))
}

/// Render mermaid source to HTML: box-drawing text with each styled run
/// wrapped in a classed `<span>` (see module docs for the class names).
pub fn render_html(src: &str, max_width: Option<usize>) -> Option<String> {
    let art = renderer::render(src, &styles(), max_width)?;
    let mut out = String::new();
    for line in &art.styled_lines {
        for span in &line.spans {
            if span.content.is_empty() {
                continue;
            }
            let text = escape_html(&span.content);
            match span.style.class {
                None => out.push_str(&text),
                Some(class) => {
                    out.push_str("<span class=\"");
                    out.push_str(class);
                    if span.style.is_italic() {
                        out.push_str(" i");
                    }
                    out.push_str("\">");
                    out.push_str(&text);
                    out.push_str("</span>");
                }
            }
        }
        out.push('\n');
    }
    Some(out)
}

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_plain_flowchart() {
        let out = render_plain("graph TD\n  A[Start] --> B[End]", Some(80)).unwrap();
        assert!(out.contains("Start"), "missing node label:\n{out}");
        assert!(out.contains('▼'), "missing arrowhead:\n{out}");
    }

    #[test]
    fn renders_classed_html() {
        let out = render_html("graph LR\n  A[a & b] -->|go| C{c}", Some(120)).unwrap();
        assert!(
            out.contains("<span class=\"e\">"),
            "missing edge span:\n{out}"
        );
        assert!(
            out.contains("<span class=\"el\">"),
            "missing edge label span:\n{out}"
        );
        assert!(out.contains("a &amp; b"), "unescaped ampersand:\n{out}");
    }

    #[test]
    fn styles_box_corners_as_edges() {
        let out = render_html("graph TD\n  A[Start] --> B[End]", Some(120)).unwrap();
        for corner in ['┌', '┐', '└', '┘'] {
            assert!(out.contains(corner), "missing box corner {corner}:\n{out}");
        }
        assert!(
            !out.contains("<span class=\"b\">"),
            "box corners must not use the border style:\n{out}"
        );
    }

    #[test]
    fn blank_input_is_none() {
        assert!(render_html("  \n ", None).is_none());
    }
}
