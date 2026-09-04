//! Frontmatter parsing for Markdown source files.

/// Markdown content after parsing optional frontmatter.
#[derive(Debug, PartialEq, Eq)]
pub struct Frontmatter<'a> {
    pub title: Option<String>,
    pub markdown: &'a str,
}

/// Parse a leading `---` frontmatter block and extract its `title` key.
pub fn parse(markdown: &str) -> Frontmatter<'_> {
    let Some(after_opening) = markdown.strip_prefix("---\n") else {
        return Frontmatter {
            title: None,
            markdown,
        };
    };
    let Some((metadata, body)) = after_opening.split_once("\n---\n") else {
        return Frontmatter {
            title: None,
            markdown,
        };
    };

    let title = metadata.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        (key.trim() == "title")
            .then(|| value.trim().trim_matches(['\'', '"']).to_string())
            .filter(|title| !title.is_empty())
    });

    Frontmatter {
        title,
        markdown: body,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_strips_title_frontmatter() {
        let parsed = parse("---\ntitle: \"Hello site\"\ndraft: false\n---\nText\n");
        assert_eq!(parsed.title.as_deref(), Some("Hello site"));
        assert_eq!(parsed.markdown, "Text\n");
    }

    #[test]
    fn preserves_markdown_without_a_complete_frontmatter_block() {
        let markdown = "---\ntitle: Unclosed\n\nText\n";
        assert_eq!(
            parse(markdown),
            Frontmatter {
                title: None,
                markdown
            }
        );
    }
}
