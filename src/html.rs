//! HTML document assembly.


/// Build a minimal semantic HTML document.
pub fn render_page(title: &str, body_html: &str, css_hrefs: &[String], md_href: &str) -> String {
    let title_esc = escape_text(title);
    let stylesheets: String = css_hrefs
        .iter()
        .map(|href| format!("<link rel=\"stylesheet\" href=\"{}\">", escape_text(href)))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title>
{stylesheets}
</head>
<body>
<header>
<p><a href="{md}">Markdown source</a></p>
</header>
<main>
{body}
</main>
</body>
</html>
"##,
        title = title_esc,
        stylesheets = stylesheets,
        md = md_href,
        body = body_html,
    )
}

/// Derive a page title from the relative markdown path.
pub fn title_from_path(rel: &std::path::Path) -> String {
    rel.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page")
        .replace('-', " ")
        .replace('_', " ")
}


fn escape_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn page_contains_source_link_and_css() {
        let html = render_page(
            "Hello",
            "<p>Hi</p>",
            &["../style.css".to_string(), "../syntax-rust.css".to_string()],
            "hello.md",
        );
        assert!(html.contains("href=\"../style.css\""));
        assert!(html.contains("href=\"../syntax-rust.css\""));
        assert!(html.contains("Markdown source"));
        assert!(html.contains("href=\"hello.md\""));
        assert!(html.contains("<main>"));
        assert!(html.contains("<p>Hi</p>"));
    }

    #[test]
    fn title_from_nested_path() {
        assert_eq!(title_from_path(Path::new("docs/my-page.md")), "my page");
    }
}
