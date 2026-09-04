//! Syntax-highlighted HTML rendering for fenced code blocks.

use syntect::highlighting::ThemeSet;
use syntect::html::{ClassStyle, ClassedHTMLGenerator, css_for_theme_with_class_style};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// Render source as syntax-highlighted HTML using CSS classes.
pub fn render_html(source: &str, language: &str) -> String {
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let syntax = syntax_set
        .find_syntax_by_token(language)
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
    let mut generator =
        ClassedHTMLGenerator::new_with_class_style(syntax, &syntax_set, ClassStyle::Spaced);

    for line in LinesWithEndings::from(source) {
        if generator
            .parse_html_for_line_which_includes_newline(line)
            .is_err()
        {
            return format!("<pre><code>{}</code></pre>", escape_html(source));
        }
    }

    format!(
        "<pre class=\"code\"><code>{}</code></pre>",
        generator.finalize()
    )
}

/// Build a responsive stylesheet matching [`render_html`]'s Syntect scope
/// classes, using bundled light and dark themes as appropriate.
pub fn stylesheet() -> String {
    let theme_set = ThemeSet::load_defaults();
    let light_theme = &theme_set.themes["Solarized (light)"];
    let dark_theme = &theme_set.themes["Solarized (dark)"];
    let light_css =
        css_for_theme_with_class_style(light_theme, ClassStyle::Spaced).unwrap_or_default();
    let dark_css =
        css_for_theme_with_class_style(dark_theme, ClassStyle::Spaced).unwrap_or_default();
    let mut css = format!("{light_css}\n@media (prefers-color-scheme: dark) {{\n{dark_css}\n}}\n");
    css.push_str(concat!(
        "\n/* Keep highlighted blocks consistent with mdsite's code and Mermaid surfaces. */\n",
        "main pre.code {\n",
        "  background-color: var(--code-bg);\n",
        "}\n",
    ));
    css
}

/// Return a stable, filesystem-safe stylesheet name for a fenced language.
pub fn stylesheet_filename(language: &str) -> String {
    let suffix: String = language
        .chars()
        .flat_map(char::to_lowercase)
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect();
    let suffix = suffix.trim_matches('-');
    format!(
        "syntax-{}.css",
        if suffix.is_empty() { "plain" } else { suffix }
    )
}

fn escape_html(source: &str) -> String {
    source
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_highlighted_rust() {
        let html = render_html("fn main() {}", "rust");
        assert!(html.contains("<pre class=\"code\">"));
        assert!(html.contains("fn"));
        assert!(html.contains("span"));
    }

    #[test]
    fn handles_unknown_languages() {
        let html = render_html("some text", "unknown-language");
        assert!(html.contains("some text"));
    }

    #[test]
    fn generates_matching_stylesheet() {
        let css = stylesheet();
        assert!(css.contains("theme \"Solarized (light)\""));
        assert!(css.contains("@media (prefers-color-scheme: dark)"));
        assert!(css.contains("theme \"Solarized (dark)\""));
        assert!(css.contains("main pre.code {\n  background-color: var(--code-bg);"));
    }

    #[test]
    fn stylesheet_filename_is_safe() {
        assert_eq!(stylesheet_filename("C++"), "syntax-c.css");
        assert_eq!(stylesheet_filename(""), "syntax-plain.css");
    }
}
