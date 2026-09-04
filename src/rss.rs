//! RSS 2.0 feed generation.

use chrono::{DateTime, Utc};

pub struct Feed<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub language: &'a str,
    pub link: &'a str,
    pub build_date: &'a DateTime<Utc>,
    pub items: Vec<Item<'a>>,
}

pub struct Item<'a> {
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub language: Option<&'a str>,
    pub link: String,
    pub publish_date: Option<&'a DateTime<Utc>>,
    pub last_updated_at: Option<&'a DateTime<Utc>>,
}

pub fn generate(feed: Feed<'_>) -> String {
    let mut xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rss version=\"2.0\">\n  <channel>\n    <title>{}</title>\n    <description>{}</description>\n    <link>{}</link>\n    <language>{}</language>\n    <lastBuildDate>{}</lastBuildDate>\n",
        escape(feed.title),
        escape(feed.description),
        escape(feed.link),
        escape(feed.language),
        rfc822_date(feed.build_date),
    );
    for item in feed.items {
        let date = item
            .last_updated_at
            .or(item.publish_date)
            .unwrap_or(feed.build_date);
        xml.push_str(&format!(
            "    <item>\n      <title>{}</title>\n      <description>{}</description>\n      <link>{}</link>\n      <guid>{}</guid>\n      <pubDate>{}</pubDate>\n",
            escape(item.title),
            escape(item.description.unwrap_or(item.title)),
            escape(&item.link),
            escape(&item.link),
            rfc822_date(date),
        ));
        if let Some(language) = item.language {
            xml.push_str(&format!(
                "      <language>{}</language>\n",
                escape(language)
            ));
        }
        xml.push_str("    </item>\n");
    }
    xml.push_str("  </channel>\n</rss>\n");
    xml
}

fn rfc822_date(date: &DateTime<Utc>) -> String {
    date.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn generates_an_rss_feed_from_item_metadata() {
        let xml = generate(Feed {
            title: "Site",
            description: "Site description",
            language: "en",
            link: "https://example.com",
            build_date: &Utc.with_ymd_and_hms(2026, 9, 4, 20, 31, 0).unwrap(),
            items: vec![Item {
                title: "First post",
                description: Some("Post description"),
                language: Some("nl"),
                link: "https://example.com/first.html".to_owned(),
                publish_date: Some(&Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).unwrap()),
                last_updated_at: Some(&Utc.with_ymd_and_hms(2026, 3, 4, 20, 31, 0).unwrap()),
            }],
        });
        assert!(xml.contains("<rss version=\"2.0\">"));
        assert!(xml.contains("<description>Post description</description>"));
        assert!(xml.contains("<language>nl</language>"));
        assert!(xml.contains("<pubDate>Wed, 04 Mar 2026 20:31:00 GMT</pubDate>"));
    }
}
