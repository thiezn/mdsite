//! HTML document assembly.

/// Build a minimal semantic HTML document.
pub fn render_page(
    title: &str,
    body_html: &str,
    css_hrefs: &[String],
    md_href: Option<&str>,
    page_rel: &std::path::Path,
) -> String {
    let title_esc = escape_text(title);
    let stylesheets: String = css_hrefs
        .iter()
        .map(|href| format!("<link rel=\"stylesheet\" href=\"{}\">", escape_text(href)))
        .collect::<Vec<_>>()
        .join("\n");
    let markdown_alternate = md_href.map_or_else(String::new, |_| {
        let markdown_path = page_rel
            .with_extension("md")
            .to_string_lossy()
            .replace('\\', "/");
        let depth = page_rel
            .parent()
            .map_or(0, |path| path.components().count());
        let href = format!("{}{}", "../".repeat(depth), markdown_path);
        format!(
            "<link rel=\"alternate\" type=\"text/markdown\" href=\"{}\">",
            escape_text(&href)
        )
    });
    let breadcrumbs = breadcrumb_html(page_rel);
    let markdown_source = md_href.map_or_else(String::new, |href| {
        format!(
            "\n<a title=\"markdown\" class=\"markdown-source\" href=\"{}\">.md</a>",
            escape_text(href)
        )
    });
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title>
{stylesheets}
{markdown_alternate}
</head>
<body>
<header>
<nav class="breadcrumbs" aria-label="Breadcrumb">{breadcrumbs}</nav>
{markdown_source}
</header>
<main>
{body}
</main>
</body>
</html>
"##,
        title = title_esc,
        stylesheets = stylesheets,
        markdown_alternate = markdown_alternate,
        breadcrumbs = breadcrumbs,
        markdown_source = markdown_source,
        body = body_html,
    )
}

fn breadcrumb_html(page_rel: &std::path::Path) -> String {
    let page_without_extension = page_rel.with_extension("");
    let components: Vec<_> = page_without_extension
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect();
    let depth = page_rel
        .parent()
        .map_or(0, |path| path.components().count());
    let mut crumbs = Vec::new();

    if components
        .first()
        .is_some_and(|component| *component != "index")
    {
        let home_href = format!("{}index.html", "../".repeat(depth));
        crumbs.push(format!("<a href=\"{home_href}\">home</a>"));
    }

    for (index, component) in components.iter().enumerate() {
        let label = if components.len() == 1 && *component == "index" {
            "home".to_string()
        } else {
            component.replace('_', " ")
        };
        let crumb = if index + 1 == components.len() {
            format!("<span aria-current=\"page\">{}</span>", escape_text(&label))
        } else {
            let target =
                std::path::Path::new(&components[..=index].join("/")).with_extension("html");
            let href = format!("{}{}", "../".repeat(depth), target.display());
            format!("<a href=\"{href}\">{}</a>", escape_text(&label))
        };
        crumbs.push(crumb);
    }

    crumbs.join("<span class=\"breadcrumb-separator\" aria-hidden=\"true\">/</span>")
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
            Some("hello.md"),
            Path::new("docs/hello.md"),
        );
        assert!(html.contains("href=\"../style.css\""));
        assert!(html.contains("href=\"../syntax-rust.css\""));
        assert!(
            html.contains("rel=\"alternate\" type=\"text/markdown\" href=\"../docs/hello.md\"")
        );
        assert!(html.contains("href=\"hello.md\""));
        assert!(html.contains("<nav class=\"breadcrumbs\""));
        assert!(html.contains("href=\"../index.html\">home</a>"));
        assert!(html.contains("href=\"../docs.html\">docs</a>"));
        assert!(html.contains("<span aria-current=\"page\">hello</span>"));
        assert!(html.contains("<main>"));
        assert!(html.contains("<p>Hi</p>"));
    }

    #[test]
    fn title_from_nested_path() {
        assert_eq!(title_from_path(Path::new("docs/my-page.md")), "my page");
    }

    #[test]
    fn root_index_breadcrumb_is_home() {
        assert_eq!(
            breadcrumb_html(Path::new("index.md")),
            "<span aria-current=\"page\">home</span>"
        );
    }
}
