//! Site generation orchestration.

use crate::css::{self, STYLE_CSS};
use crate::error::Result;
use crate::frontmatter;
use crate::html;
use crate::markdown::{extract_code_blocks, markdown_to_html};
use crate::mermaid;
use crate::syntax;
use crate::walk::{MdFile, collect_asset_files, collect_markdown_files};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

/// Build a static site from Markdown files under `input` into `output`.
///
/// For each `.md` file:
/// - emit a matching `.html` page (folder structure preserved),
/// - copy the `.md` next to the `.html`,
/// - copy all non-Markdown files unchanged (folder structure preserved),
/// - render Mermaid fences as styled HTML diagrams,
/// - render other fenced blocks with syntax-specific stylesheets.
///
/// The output directory is cleared before generation, then shared stylesheets
/// are written at its root.
pub fn build(input: &Path, output: &Path) -> Result<()> {
    if output.exists() {
        fs::remove_dir_all(output)?;
    }
    fs::create_dir_all(output)?;

    let files = collect_markdown_files(input)?;
    for asset in collect_asset_files(input)? {
        let destination = output.join(&asset.relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(asset.absolute, destination)?;
    }
    fs::write(output.join("style.css"), STYLE_CSS)?;
    let mut syntax_stylesheets = BTreeSet::new();
    for file in &files {
        let markdown = fs::read_to_string(&file.absolute)?;
        for block in extract_code_blocks(frontmatter::parse(&markdown).markdown).blocks {
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
    generate_directory_pages(&files, output)?;
    Ok(())
}

fn convert_file(file: &MdFile, output_root: &Path) -> Result<()> {
    let md_bytes = fs::read(&file.absolute)?;
    let md_text = String::from_utf8(md_bytes)?;
    let parsed = frontmatter::parse(&md_text);
    let title = parsed
        .title
        .as_deref()
        .map(str::to_owned)
        .unwrap_or_else(|| html::title_from_path(&file.relative));
    let markdown = match parsed.title.as_deref() {
        Some(_) => format!("# {title}\n\n{}", parsed.markdown),
        None => parsed.markdown.to_owned(),
    };

    let html_rel = file.relative.with_extension("html");
    let out_dir = match html_rel.parent() {
        Some(p) if !p.as_os_str().is_empty() => output_root.join(p),
        _ => output_root.to_path_buf(),
    };
    fs::create_dir_all(&out_dir)?;

    let prepared = extract_code_blocks(&markdown);
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
    let page = html::render_page(&title, &body, &css_hrefs, Some(md_href), &file.relative);

    let html_path = output_root.join(&html_rel);
    fs::write(&html_path, page)?;

    let md_out = out_dir.join(
        file.relative
            .file_name()
            .expect("markdown file has a file name"),
    );
    fs::write(&md_out, markdown)?;

    Ok(())
}

fn generate_directory_pages(files: &[MdFile], output_root: &Path) -> Result<()> {
    let mut files_by_directory = BTreeMap::<_, Vec<_>>::new();
    for file in files {
        let Some(directory) = file
            .relative
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        else {
            continue;
        };
        files_by_directory
            .entry(directory.to_path_buf())
            .or_default()
            .push(file);
    }

    for (directory, direct_files) in files_by_directory {
        let folder_name = directory
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("page");
        let custom_page = directory
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(format!("{folder_name}.md"));
        if files.iter().any(|file| file.relative == custom_page) {
            continue;
        }

        let title = folder_name.replace('_', " ");
        let mut markdown = format!("# {title}\n");
        for file in direct_files {
            let filename = file
                .relative
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("page");
            markdown.push_str(&format!(
                "\n- [{filename}]({}/{filename}.html)\n",
                directory.display()
            ));
        }

        let html_rel = directory.with_extension("html");
        let body = markdown_to_html(&markdown);
        let css_hrefs = [css::relative_css_href(&html_rel)];
        let md_rel = directory.with_extension("md");
        let md_href = md_rel.file_name().and_then(|name| name.to_str());
        let page = html::render_page(&title, &body, &css_hrefs, md_href, &html_rel);
        fs::write(output_root.join(html_rel), page)?;
        fs::write(output_root.join(md_rel), markdown)?;

        let source_index = directory.join("index.md");
        if !files.iter().any(|file| file.relative == source_index) {
            let legacy_index = output_root.join(&directory).join("index.html");
            if legacy_index.exists() {
                fs::remove_file(legacy_index)?;
            }
        }
    }

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
        assert!(index.contains("rel=\"alternate\" type=\"text/markdown\" href=\"index.md\""));
        assert!(index.contains("href=\"index.md\""));
        assert!(index.contains("<em>site</em>"));

        let nested = fs::read_to_string(output.path().join("nested/page.html")).unwrap();
        assert!(nested.contains("href=\"../style.css\""));
        assert!(nested.contains("href=\"page.md\""));
        assert!(nested.contains("href=\"../index.html\">home</a>"));
        assert!(nested.contains("href=\"../nested.html\">nested</a>"));
    }

    #[test]
    fn build_removes_stale_destination_files() {
        let input = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        fs::write(input.path().join("index.md"), "# Current site\n").unwrap();
        fs::write(output.path().join("old.html"), "stale page").unwrap();
        fs::create_dir_all(output.path().join("old/nested")).unwrap();
        fs::write(output.path().join("old/nested/file.txt"), "stale file").unwrap();

        build(input.path(), output.path()).unwrap();

        assert!(output.path().join("index.html").is_file());
        assert!(output.path().join("style.css").is_file());
        assert!(!output.path().join("old.html").exists());
        assert!(!output.path().join("old").exists());
    }

    #[test]
    fn build_copies_non_markdown_files_preserving_paths_and_contents() {
        let input = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        fs::create_dir_all(input.path().join("images/icons")).unwrap();
        fs::write(input.path().join("index.md"), "# Home\n").unwrap();
        fs::write(input.path().join("robots.txt"), "User-agent: *\n").unwrap();
        fs::write(input.path().join("images/icons/logo.png"), [0, 1, 2, 255]).unwrap();

        build(input.path(), output.path()).unwrap();

        assert_eq!(
            fs::read_to_string(output.path().join("robots.txt")).unwrap(),
            "User-agent: *\n"
        );
        assert_eq!(
            fs::read(output.path().join("images/icons/logo.png")).unwrap(),
            [0, 1, 2, 255]
        );
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

    #[test]
    fn build_uses_frontmatter_and_generates_directory_pages() {
        let input = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        fs::create_dir_all(input.path().join("project_notes")).unwrap();
        fs::create_dir_all(input.path().join("custom")).unwrap();
        fs::write(
            input.path().join("project_notes/guide.md"),
            "---\ntitle: Getting started\n---\nThis is the guide.\n",
        )
        .unwrap();
        fs::write(input.path().join("project_notes/faq.md"), "Questions.\n").unwrap();
        fs::write(input.path().join("custom/item.md"), "Custom item.\n").unwrap();
        fs::write(input.path().join("custom.md"), "# Custom landing page\n").unwrap();
        fs::create_dir_all(output.path().join("project_notes")).unwrap();
        fs::write(
            output.path().join("project_notes/index.html"),
            "legacy index",
        )
        .unwrap();

        build(input.path(), output.path()).unwrap();

        let guide = fs::read_to_string(output.path().join("project_notes/guide.html")).unwrap();
        assert!(guide.contains("<title>Getting started</title>"));
        assert!(guide.contains("<h1>Getting started</h1>"));
        assert!(!guide.contains("title: Getting started"));
        assert_eq!(
            fs::read_to_string(output.path().join("project_notes/guide.md")).unwrap(),
            "# Getting started\n\nThis is the guide.\n"
        );

        let directory_page = fs::read_to_string(output.path().join("project_notes.html")).unwrap();
        assert!(directory_page.contains("<title>project notes</title>"));
        assert!(directory_page.contains("<h1>project notes</h1>"));
        assert!(directory_page.contains("href=\"project_notes/guide.html\""));
        assert!(directory_page.contains("href=\"project_notes/faq.html\""));
        assert!(directory_page.contains("href=\"index.html\">home</a>"));
        assert!(directory_page.contains("<span aria-current=\"page\">project notes</span>"));
        assert_eq!(
            fs::read_to_string(output.path().join("project_notes.md")).unwrap(),
            "# project notes\n\n- [faq](project_notes/faq.html)\n\n- [guide](project_notes/guide.html)\n"
        );
        let guide = fs::read_to_string(output.path().join("project_notes/guide.html")).unwrap();
        assert!(guide.contains(
            "rel=\"alternate\" type=\"text/markdown\" href=\"../project_notes/guide.md\""
        ));
        assert!(guide.contains("href=\"../project_notes.html\">project notes</a>"));
        assert!(!output.path().join("project_notes/index.html").exists());
        let custom_page = fs::read_to_string(output.path().join("custom.html")).unwrap();
        assert!(custom_page.contains("<h1>Custom landing page</h1>"));
        assert!(!custom_page.contains("href=\"custom/item.html\""));
    }
}
