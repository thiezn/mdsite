//! Shared stylesheet for generated sites.

/// Minimal clean CSS written to the output root as `style.css`.
pub const STYLE_CSS: &str = r#"/* mdsite - minimal clean styles */
:root {
  color-scheme: light dark;
  --fg: #1a1a1a;
  --bg: #fafafa;
  --link: #0645ad;
  --muted: color-mix(in srgb, var(--fg) 55%, var(--bg));
  --border: color-mix(in srgb, var(--fg) 14%, var(--bg));
  --code-bg: color-mix(in srgb, var(--fg) 6%, var(--bg));
  --focus: color-mix(in srgb, var(--link) 65%, var(--bg));
}
@media (prefers-color-scheme: dark) {
  :root {
    --fg: #e8e8e8;
    --bg: #1a1a1a;
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
  overflow-wrap: break-word;
}
:focus-visible {
  outline: 2px solid var(--focus);
  outline-offset: 2px;
}
header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.125rem 0;
  a {
    color: var(--muted);
    font-size: 0.9rem;
    text-decoration: none;
    &:hover { text-decoration: underline; }
  }
}
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
main {
  h1, h2, h3, h4, h5, h6 {
    line-height: 1.25;
    margin-top: 1.5em;
    margin-bottom: 0.5em;
    text-wrap: balance;
    scroll-margin-top: 1.25rem;
  }
  > h1:first-child { margin-top: 0; }
  p {
    margin: 0.75em 0;
    text-wrap: pretty;
  }
  a { color: var(--link); }
  ul, ol { padding-left: 1.5rem; }
  li { margin: 0.25em 0; }
  blockquote {
    margin: 1rem 0;
    padding: 0.25rem 0 0.25rem 1rem;
    border-left: 3px solid var(--border);
    color: var(--muted);
  }
  hr {
    border: none;
    border-top: 1px solid var(--border);
    margin: 2rem 0;
  }
  img, svg {
    max-width: 100%;
    height: auto;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    margin: 1rem 0;
    font-size: 0.95em;
  }
  th, td {
    border: 1px solid var(--border);
    padding: 0.4rem 0.65rem;
    text-align: left;
  }
  th {
    background: var(--code-bg);
    font-weight: 600;
  }
  code {
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    font-size: 0.9em;
    background: var(--code-bg);
    padding: 0.1em 0.35em;
    border-radius: 3px;
    overflow-wrap: anywhere;
  }
  pre {
    background: var(--code-bg);
    padding: 1rem;
    overflow-x: auto;
    border-radius: 4px;
    border: 1px solid var(--border);
    tab-size: 2;
    code {
      background: none;
      padding: 0;
      font-size: 0.875em;
      overflow-wrap: normal;
    }
    &.mermaid {
      display: block;
      max-width: 100%;
      margin: 1.25rem 0;
      white-space: pre;
      line-height: 1.2;
      .b { color: var(--muted); }
      .n { color: var(--fg); }
      .e { color: var(--link); }
      .el { color: var(--muted); }
      .t { color: var(--fg); font-weight: 700; }
      .i { font-style: italic; }
    }
  }
}
.page-metadata {
  margin-top: 2rem;
  display: flex;
  align-items: start;
  justify-content: space-between;
  gap: 1rem;
  color: var(--muted);
  font-size: 0.9rem;
  line-height: 1.5;
}
.page-footer { text-align: left; }
.page-footer a { color: inherit; }
.page-dates { margin-left: auto; text-align: right; }
.directory-list {
  margin: 1rem 0;
  padding: 0;
  list-style: none;
}
.directory-entry {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 1.5rem;
  padding: 0.65rem 0;
  border-bottom: 1px solid var(--border);
}
.directory-description {
  margin: 0.15rem 0 0;
  color: var(--muted);
  font-size: 0.9rem;
}
.directory-date {
  flex: none;
  color: var(--muted);
  font-size: 0.9rem;
  text-align: right;
}
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }
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
