//! Frontmatter parsing for Markdown source files.

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

/// Markdown content and metadata after parsing optional frontmatter.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Frontmatter<'a> {
    pub title: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub publish_date: Option<DateTime<Utc>>,
    pub last_updated_at: Option<DateTime<Utc>>,
    pub footer: Option<String>,
    pub include_in_rss: bool,
    pub include_in_sitemap: bool,
    pub markdown: &'a str,
}

/// Parse a leading `---` frontmatter block.
pub fn parse(markdown: &str) -> Frontmatter<'_> {
    let Some(after_opening) = markdown.strip_prefix("---\n") else {
        return Frontmatter::without_metadata(markdown);
    };
    let Some((metadata, body)) = after_opening.split_once("\n---\n") else {
        return Frontmatter::without_metadata(markdown);
    };

    Frontmatter {
        title: value(metadata, "title"),
        description: value(metadata, "description"),
        language: value(metadata, "language"),
        publish_date: value(metadata, "publish_date").and_then(|date| parse_datetime(&date)),
        last_updated_at: value(metadata, "last_updated_at").and_then(|date| parse_datetime(&date)),
        footer: value(metadata, "footer"),
        include_in_rss: boolean(metadata, "include_in_rss").unwrap_or(true),
        include_in_sitemap: boolean(metadata, "include_in_sitemap").unwrap_or(true),
        markdown: body,
    }
}

impl<'a> Frontmatter<'a> {
    fn without_metadata(markdown: &'a str) -> Self {
        Self {
            title: None,
            description: None,
            language: None,
            publish_date: None,
            last_updated_at: None,
            footer: None,
            include_in_rss: true,
            include_in_sitemap: true,
            markdown,
        }
    }
}

fn value(metadata: &str, key: &str) -> Option<String> {
    metadata.lines().find_map(|line| {
        let (candidate, value) = line.split_once(':')?;
        (candidate.trim() == key)
            .then(|| value.trim().trim_matches(['\'', '"']).to_string())
            .filter(|value| !value.is_empty())
    })
}

fn boolean(metadata: &str, key: &str) -> Option<bool> {
    value(metadata, key).and_then(|value| match value.as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    })
}

fn parse_datetime(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|date| date.with_timezone(&Utc))
        .ok()
        .or_else(|| {
            ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%dT%H:%M"]
                .iter()
                .find_map(|format| NaiveDateTime::parse_from_str(value, format).ok())
                .map(|date| DateTime::from_naive_utc_and_offset(date, Utc))
        })
        .or_else(|| {
            NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .ok()?
                .and_hms_opt(0, 0, 0)
                .map(|date| DateTime::from_naive_utc_and_offset(date, Utc))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_strips_title_frontmatter() {
        let parsed = parse("---\ntitle: \"Hello site\"\ndraft: false\n---\nText\n");
        assert_eq!(parsed.title.as_deref(), Some("Hello site"));
        assert!(parsed.include_in_rss);
        assert!(parsed.include_in_sitemap);
        assert_eq!(parsed.markdown, "Text\n");
    }

    #[test]
    fn parses_page_metadata_and_inclusion_flags() {
        let parsed = parse(
            "---\ntitle: Page\ndescription: \"Summary\"\nlanguage: nl\npublish_date: 2026-01-02\nlast_updated_at: 2026-03-04\nfooter: \"Made with *care*.\"\ninclude_in_rss: false\ninclude_in_sitemap: false\n---\nText\n",
        );
        assert_eq!(parsed.description.as_deref(), Some("Summary"));
        assert_eq!(parsed.language.as_deref(), Some("nl"));
        assert_eq!(
            parsed.publish_date.unwrap().to_rfc3339(),
            "2026-01-02T00:00:00+00:00"
        );
        assert_eq!(
            parsed.last_updated_at.unwrap().to_rfc3339(),
            "2026-03-04T00:00:00+00:00"
        );
        assert_eq!(parsed.footer.as_deref(), Some("Made with *care*."));
        assert!(!parsed.include_in_rss);
        assert!(!parsed.include_in_sitemap);
    }

    #[test]
    fn preserves_markdown_without_a_complete_frontmatter_block() {
        let markdown = "---\ntitle: Unclosed\n\nText\n";
        assert_eq!(
            parse(markdown),
            Frontmatter {
                title: None,
                description: None,
                language: None,
                publish_date: None,
                last_updated_at: None,
                footer: None,
                include_in_rss: true,
                include_in_sitemap: true,
                markdown
            }
        );
    }

    #[test]
    fn parses_partial_iso8601_dates_as_utc() {
        let parsed = parse(
            "---\npublish_date: 2026-09-04T20:31\nlast_updated_at: 2026-09-04T22:31:00+02:00\n---\nText\n",
        );
        assert_eq!(
            parsed.publish_date.unwrap().to_rfc3339(),
            "2026-09-04T20:31:00+00:00"
        );
        assert_eq!(
            parsed.last_updated_at.unwrap().to_rfc3339(),
            "2026-09-04T20:31:00+00:00"
        );
    }
}
