//! Site generation orchestration.

use crate::css::{self, STYLE_CSS};
use crate::error::Result;
use crate::html;
use crate::markdown::{extract_mermaid, markdown_to_html};
use crate::mermaid;
use crate::walk::{collect_markdown_files, MdFile};
use std::fs;
use std::path::Path;

/// Build a static site from Markdown files under `input` into `output`.
///
/// For each `.md` file:
/// - emit a matching `.html` page (folder structure preserved),
/// - copy the `.md` next to the `.html`,
/// - render Mermaid fences to sibling SVG images (requires `mmdc`).
///
/// Writes a shared `style.css` at the output root.
pub fn build(input: &Path, output: &Path) -> Result<()> {
    fs::create_dir_all(output)?;
    fs::write(output.join("style.css"), STYLE_CSS)?;

    let files = collect_markdown_files(input)?;
    for file in &files {
        convert_file(file, output)?;
    }
    Ok(())
}

fn convert_file(file: &MdFile, output_root: &Path) -> Result<()> {
    let md_bytes = fs::read(&file.absolute)?;
    let md_text = String::from_utf8(md_bytes)?;

    let stem = file
        .relative
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page");

    let html_rel = file.relative.with_extension("html");
    let out_dir = match html_rel.parent() {
        Some(p) if !p.as_os_str().is_empty() => output_root.join(p),
        _ => output_root.to_path_buf(),
    };
    fs::create_dir_all(&out_dir)?;

    let prepared = extract_mermaid(&md_text, stem);
    mermaid::render_blocks(&prepared.mermaid, &out_dir, stem)?;

    let body = markdown_to_html(&prepared.markdown);
    let title = html::title_from_path(&file.relative);
    let css_href = css::relative_css_href(&html_rel);
    let md_href = file
        .relative
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("source.md");
    let page = html::render_page(&title, &body, &css_href, md_href);

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
        assert!(index.contains("Markdown source"));
        assert!(index.contains("href=\"index.md\""));
        assert!(index.contains("<em>site</em>"));

        let nested = fs::read_to_string(output.path().join("nested/page.html")).unwrap();
        assert!(nested.contains("href=\"../style.css\""));
        assert!(nested.contains("href=\"page.md\""));
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
}
