//! RSS 2.0 feed generation.

pub struct Feed<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub language: &'a str,
    pub link: &'a str,
    pub build_date: &'a str,
    pub items: Vec<Item<'a>>,
}

pub struct Item<'a> {
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub language: Option<&'a str>,
    pub link: String,
    pub publish_date: Option<&'a str>,
    pub last_updated_at: Option<&'a str>,
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
            xml.push_str(&format!("      <language>{}</language>\n", escape(language)));
        }
        xml.push_str("    </item>\n");
    }
    xml.push_str("  </channel>\n</rss>\n");
    xml
}

fn rfc822_date(date: &str) -> String {
    let Some((year, month, day)) = parse_date(date) else {
        return date.to_owned();
    };
    let weekday = weekday(year, month, day);
    let month_name = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ][month as usize - 1];
    format!("{weekday}, {day:02} {month_name} {year:04} 00:00:00 GMT")
}

fn parse_date(date: &str) -> Option<(i64, u32, u32)> {
    let mut parts = date.split('-');
    let (Some(year), Some(month), Some(day), None) = (parts.next(), parts.next(), parts.next(), parts.next()) else {
        return None;
    };
    Some((year.parse().ok()?, month.parse().ok()?, day.parse().ok()?))
}

fn weekday(year: i64, month: u32, day: u32) -> &'static str {
    let month = if month < 3 { month + 12 } else { month } as i64;
    let year = if month > 12 { year - 1 } else { year };
    let day_of_week = (day as i64 + 13 * (month + 1) / 5 + year + year / 4 - year / 100 + year / 400) % 7;
    ["Sat", "Sun", "Mon", "Tue", "Wed", "Thu", "Fri"][day_of_week as usize]
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

    #[test]
    fn generates_an_rss_feed_from_item_metadata() {
        let xml = generate(Feed {
            title: "Site",
            description: "Site description",
            language: "en",
            link: "https://example.com",
            build_date: "2026-09-04",
            items: vec![Item {
                title: "First post",
                description: Some("Post description"),
                language: Some("nl"),
                link: "https://example.com/first.html".to_owned(),
                publish_date: Some("2026-01-02"),
                last_updated_at: Some("2026-03-04"),
            }],
        });
        assert!(xml.contains("<rss version=\"2.0\">"));
        assert!(xml.contains("<description>Post description</description>"));
        assert!(xml.contains("<language>nl</language>"));
        assert!(xml.contains("<pubDate>Wed, 04 Mar 2026 00:00:00 GMT</pubDate>"));
    }
}