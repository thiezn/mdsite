//! Site generation orchestration.

use crate::css::{self, STYLE_CSS};
use crate::error::Result;
use crate::html;
use crate::markdown::{extract_code_blocks, markdown_to_html};
use crate::mermaid;
use crate::syntax;
use crate::walk::{MdFile, collect_markdown_files};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Build a static site from Markdown files under `input` into `output`.
///
/// For each `.md` file:
/// - emit a matching `.html` page (folder structure preserved),
/// - copy the `.md` next to the `.html`,
/// - render Mermaid fences as styled HTML diagrams,
/// - render other fenced blocks with syntax-specific stylesheets.
///
/// Writes shared stylesheets at the output root.
pub fn build(input: &Path, output: &Path) -> Result<()> {
    fs::create_dir_all(output)?;
    fs::write(output.join("style.css"), STYLE_CSS)?;

    let files = collect_markdown_files(input)?;
    let mut syntax_stylesheets = BTreeSet::new();
    for file in &files {
        let markdown = fs::read_to_string(&file.absolute)?;
        for block in extract_code_blocks(&markdown).blocks {
            if !block.language.eq_ignore_ascii_case("mermaid") {
                syntax_stylesheets.insert(syntax::stylesheet_filename(&block.language));
            }
        }
    }
    if !syntax_stylesheets.is_empty() {
        let stylesheet = syntax::stylesheet();
        for filename in syntax_stylesheets {
            fs::write(output.join(filename), &stylesheet)?;
        }
    }

    for file in &files {
        convert_file(file, output)?;
    }
    Ok(())
}

fn convert_file(file: &MdFile, output_root: &Path) -> Result<()> {
    let md_bytes = fs::read(&file.absolute)?;
    let md_text = String::from_utf8(md_bytes)?;

    let html_rel = file.relative.with_extension("html");
    let out_dir = match html_rel.parent() {
        Some(p) if !p.as_os_str().is_empty() => output_root.join(p),
        _ => output_root.to_path_buf(),
    };
    fs::create_dir_all(&out_dir)?;

    let prepared = extract_code_blocks(&md_text);
    let mut body = markdown_to_html(&prepared.markdown);
    let mut syntax_stylesheets = BTreeSet::new();
    for block in &prepared.blocks {
        let placeholder = format!(
            "<pre class=\"code-placeholder\" data-index=\"{}\"></pre>",
            block.index
        );
        let rendered = if block.language.eq_ignore_ascii_case("mermaid") {
            let diagram = mermaid::render_html(&block.source, Some(120)).unwrap_or_default();
            format!("<pre class=\"mermaid\">{diagram}</pre>")
        } else {
            syntax_stylesheets.insert(syntax::stylesheet_filename(&block.language));
            syntax::render_html(&block.source, &block.language)
        };
        body = body.replace(&placeholder, &rendered);
    }
    let title = html::title_from_path(&file.relative);
    let mut css_hrefs = vec![css::relative_css_href(&html_rel)];
    css_hrefs.extend(
        syntax_stylesheets
            .iter()
            .map(|filename| css::relative_asset_href(&html_rel, filename)),
    );
    let md_href = file
        .relative
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("source.md");
    let page = html::render_page(&title, &body, &css_hrefs, md_href, &file.relative);

    let html_path = output_root.join(&html_rel);
    fs::write(&html_path, page)?;

    let md_out = out_dir.join(
        file.relative
            .file_name()
            .expect("markdown file has a file name"),
    );
    fs::write(&md_out, md_text)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn build_basic_site_without_mermaid() {
        let input = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        fs::create_dir_all(input.path().join("nested")).unwrap();
        fs::write(
            input.path().join("index.md"),
            "# Hello\n\nA *site* with **bold** and `code`.\n\n- one\n- two\n",
        )
        .unwrap();
        fs::write(
            input.path().join("nested/page.md"),
            "## Nested\n\n[home](../index.md)\n",
        )
        .unwrap();

        build(input.path(), output.path()).unwrap();

        assert!(output.path().join("style.css").is_file());
        assert!(output.path().join("index.html").is_file());
        assert!(output.path().join("index.md").is_file());
        assert!(output.path().join("nested/page.html").is_file());
        assert!(output.path().join("nested/page.md").is_file());

        let index = fs::read_to_string(output.path().join("index.html")).unwrap();
        assert!(index.contains("href=\"style.css\""));
        assert!(index.contains(">markdown</a>"));
        assert!(index.contains("href=\"index.md\""));
        assert!(index.contains("<em>site</em>"));

        let nested = fs::read_to_string(output.path().join("nested/page.html")).unwrap();
        assert!(nested.contains("href=\"../style.css\""));
        assert!(nested.contains("href=\"page.md\""));
        assert!(nested.contains("href=\"../index.html\">home</a>"));
        assert!(nested.contains("<span>nested</span>"));
    }

    #[test]
    fn input_must_be_directory() {
        let f = tempfile::NamedTempFile::new().unwrap();
        let output = tempfile::tempdir().unwrap();
        let err = build(f.path(), output.path()).unwrap_err();
        match err {
            crate::error::Error::InputNotDirectory(p) => {
                assert_eq!(p, PathBuf::from(f.path()));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn build_renders_mermaid_as_embedded_html() {
        let input = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        fs::write(
            input.path().join("index.md"),
            "# Diagram\n\n```mermaid\nflowchart LR\n  A[Start] --> B[End]\n```\n",
        )
        .unwrap();

        build(input.path(), output.path()).unwrap();

        let index = fs::read_to_string(output.path().join("index.html")).unwrap();
        assert!(index.contains("<pre class=\"mermaid\">"));
        assert!(index.contains("Start"));
        assert!(index.contains("class=\"b\""));
        assert!(!index.contains("-mermaid-1.svg"));
        assert!(!output.path().join("index-mermaid-1.svg").exists());
    }

    #[test]
    fn build_renders_code_blocks_with_syntax_highlighting() {
        let input = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        fs::write(
            input.path().join("index.md"),
            "# Code\n\n```rust\nfn main() {}\n```\n\n```python\nprint('hello')\n```\n",
        )
        .unwrap();

        build(input.path(), output.path()).unwrap();

        let index = fs::read_to_string(output.path().join("index.html")).unwrap();
        assert!(index.contains("fn"));
        assert!(index.contains("<pre class=\"code\">"));
        assert!(index.contains("href=\"syntax-rust.css\""));
        assert!(index.contains("href=\"syntax-python.css\""));
        assert!(output.path().join("syntax-rust.css").is_file());
        assert!(output.path().join("syntax-python.css").is_file());
    }
}
