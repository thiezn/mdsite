//! Shared stylesheet for generated sites.

/// Minimal clean CSS written to the output root as `style.css`.
pub const STYLE_CSS: &str = r#"/* mdsite - minimal clean styles */
:root {
  color-scheme: light dark;
  --fg: #1a1a1a;
  --bg: #fafafa;
  --muted: #555;
  --border: #ddd;
  --code-bg: #f0f0f0;
  --link: #0645ad;
}
@media (prefers-color-scheme: dark) {
  :root {
    --fg: #e8e8e8;
    --bg: #1a1a1a;
    --muted: #aaa;
    --border: #333;
    --code-bg: #2a2a2a;
    --link: #8ab4f8;
  }
}
* { box-sizing: border-box; }
body {
  margin: 0 auto;
  max-width: 48rem;
  padding: 1.5rem 1.25rem 3rem;
  font-family: system-ui, -apple-system, Segoe UI, Roboto, sans-serif;
  line-height: 1.6;
  color: var(--fg);
  background: var(--bg);
}
header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.125rem 0;
}
header a {
  color: var(--muted);
  font-size: 0.9rem;
  text-decoration: none;
}
header a:hover { text-decoration: underline; }
.breadcrumbs {
  display: flex;
  min-width: 0;
  gap: 0.4rem;
  overflow-x: auto;
  white-space: nowrap;
  color: var(--muted);
  font-size: 0.9rem;
}
.breadcrumb-separator { color: var(--border); }
.markdown-source { flex: none; }
main h1, main h2, main h3, main h4, main h5, main h6 {
  line-height: 1.25;
  margin-top: 1.5em;
  margin-bottom: 0.5em;
}
main > h1:first-child { margin-top: 0; }
main p { margin: 0.75em 0; }
main a { color: var(--link); }
.page-metadata {
  margin-top: 2rem;
  color: var(--muted);
  font-size: 0.9rem;
  line-height: 1.5;
  text-align: right;
}
main ul, main ol { padding-left: 1.5rem; }
main li { margin: 0.25em 0; }
main code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 0.9em;
  background: var(--code-bg);
  padding: 0.1em 0.35em;
  border-radius: 3px;
}
main pre {
  background: var(--code-bg);
  padding: 1rem;
  overflow-x: auto;
  border-radius: 4px;
  border: 1px solid var(--border);
}
main pre code {
  background: none;
  padding: 0;
  font-size: 0.875em;
}
main pre.mermaid {
  display: block;
  max-width: 100%;
  margin: 1.25rem 0;
  white-space: pre;
  line-height: 1.2;
}
main pre.mermaid .b { color: var(--muted); }
main pre.mermaid .n { color: var(--fg); }
main pre.mermaid .e { color: var(--link); }
main pre.mermaid .el { color: var(--muted); }
main pre.mermaid .t { color: var(--fg); font-weight: 700; }
main pre.mermaid .i { font-style: italic; }
}
"#;

/// Relative path from an HTML page (relative to output root) to `style.css`.
pub fn relative_css_href(page_rel: &std::path::Path) -> String {
    relative_asset_href(page_rel, "style.css")
}

/// Relative path from an HTML page to an output-root asset.
pub fn relative_asset_href(page_rel: &std::path::Path, asset: &str) -> String {
    let depth = page_rel
        .parent()
        .map(|p| p.components().count())
        .unwrap_or(0);
    if depth == 0 {
        asset.to_string()
    } else {
        format!("{}{}", "../".repeat(depth), asset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn css_href_root() {
        assert_eq!(relative_css_href(Path::new("index.html")), "style.css");
    }

    #[test]
    fn css_href_nested() {
        assert_eq!(
            relative_css_href(Path::new("docs/guide.html")),
            "../style.css"
        );
        assert_eq!(
            relative_css_href(Path::new("a/b/c.html")),
            "../../style.css"
        );
    }

    #[test]
    fn asset_href_nested() {
        assert_eq!(
            relative_asset_href(Path::new("docs/guide.html"), "syntax-rust.css"),
            "../syntax-rust.css"
        );
    }
}
