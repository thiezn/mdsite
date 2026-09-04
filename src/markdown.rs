//! Markdown parsing helpers (Mermaid extraction + HTML conversion).

use pulldown_cmark::{Event, Options, Parser};

/// A fenced Mermaid diagram extracted from Markdown.
#[derive(Debug, Clone)]
pub struct MermaidBlock {
    pub index: usize,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct PreparedMarkdown {
    pub markdown: String,
    pub mermaid: Vec<MermaidBlock>,
}

pub fn extract_mermaid(markdown: &str, stem: &str) -> PreparedMarkdown {
    let mut out = String::with_capacity(markdown.len());
    let mut mermaid = Vec::new();
    let mut lines = markdown.lines().peekable();
    let mut index = 0usize;

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            let lang = trimmed.trim_start_matches("```").trim();
            if lang.eq_ignore_ascii_case("mermaid") {
                index += 1;
                let mut body = String::new();
                while let Some(inner) = lines.next() {
                    if inner.trim().starts_with("```") {
                        break;
                    }
                    if !body.is_empty() {
                        body.push('\n');
                    }
                    body.push_str(inner);
                }
                let filename = format!("{stem}-mermaid-{index}.svg");
                out.push_str(&format!(
                    "\n<img class=\"mermaid\" src=\"{filename}\" alt=\"Mermaid diagram {index}\" />\n\n"
                ));
                mermaid.push(MermaidBlock {
                    index,
                    source: body,
                });
                continue;
            }
            out.push_str(line);
            out.push('\n');
            while let Some(inner) = lines.next() {
                out.push_str(inner);
                out.push('\n');
                if inner.trim().starts_with("```") {
                    break;
                }
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }

    PreparedMarkdown {
        markdown: out,
        mermaid,
    }
}

/// Convert Markdown to an HTML fragment (no surrounding document).
/// Lines that are already Mermaid <img> tags are passed through as raw HTML.
pub fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let mut html = String::new();
    let mut md_buf = String::new();

    for line in markdown.lines() {
        let t = line.trim();
        if t.starts_with("<img class=\"mermaid\"") && t.ends_with("/>") {
            if !md_buf.is_empty() {
                let parser = Parser::new_ext(&md_buf, options);
                pulldown_cmark::html::push_html(&mut html, parser);
                md_buf.clear();
            }
            html.push_str(t);
            html.push('\n');
        } else {
            md_buf.push_str(line);
            md_buf.push('\n');
        }
    }
    if !md_buf.is_empty() {
        let parser = Parser::new_ext(&md_buf, options);
        pulldown_cmark::html::push_html(&mut html, parser);
    }
    let _ = Event::SoftBreak;
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_mermaid_and_keeps_code() {
        let md = "# Hi\n\n```mermaid\ngraph TD; A-->B;\n```\n\n```rust\nfn main() {}\n```\n";
        let prep = extract_mermaid(md, "page");
        assert_eq!(prep.mermaid.len(), 1);
        assert!(prep.mermaid[0].source.contains("graph TD"));
        assert!(prep.markdown.contains("page-mermaid-1.svg"));
        assert!(prep.markdown.contains("```rust"));
        assert!(!prep.markdown.contains("```mermaid"));
    }

    #[test]
    fn converts_basic_markdown() {
        let html = markdown_to_html(
            "# Title\n\nA *em* and **strong** and `code`.\n\n- a\n- b\n\n1. one\n2. two\n\n[link](https://example.com)\n",
        );
        assert!(html.contains("<h1>"));
        assert!(html.contains("<em>em</em>"));
        assert!(html.contains("<strong>strong</strong>"));
        assert!(html.contains("<code>code</code>"));
        assert!(html.contains("<ul>"));
        assert!(html.contains("<ol>"));
        assert!(html.contains("href=\"https://example.com\""));
    }

    #[test]
    fn fenced_code_block() {
        let html = markdown_to_html("```\nplain\n```\n");
        assert!(html.contains("<pre>"));
        assert!(html.contains("<code>"));
        assert!(html.contains("plain"));
    }
}
