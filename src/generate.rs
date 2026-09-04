//! Site generation orchestration.

use crate::css::{self, STYLE_CSS};
use crate::error::Result;
use crate::frontmatter;
use crate::html;
use crate::markdown::{extract_code_blocks, markdown_to_html};
use crate::mermaid;
use crate::rss;
use crate::syntax;
use crate::walk::{MdFile, collect_asset_files, collect_markdown_files};
use chrono::{DateTime, SecondsFormat, Utc};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

struct SiteConfig {
    domain: String,
    llms_prefix: Option<String>,
    generate_sitemap: bool,
    generate_llms: bool,
    generate_rss: bool,
    generate_robots: bool,
}

struct PageMetadata {
    title: Option<String>,
    description: Option<String>,
    language: Option<String>,
    publish_date: Option<DateTime<Utc>>,
    last_updated_at: Option<DateTime<Utc>>,
    include_in_rss: bool,
    include_in_sitemap: bool,
}

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
    let files = collect_markdown_files(input)?;
    let config = read_config(input)?;
    if output.exists() {
        fs::remove_dir_all(output)?;
    }
    fs::create_dir_all(output)?;

    for asset in collect_asset_files(input)? {
        if asset.relative == Path::new("mdsite.toml") {
            continue;
        }
        let destination = output.join(&asset.relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(asset.absolute, destination)?;
    }
    generate_robots(input, output, &config)?;
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
        convert_file(file, output, config.generate_rss)?;
    }
    let directory_pages = generate_directory_pages(&files, output, config.generate_rss)?;
    generate_site_metadata(&files, &directory_pages, output, &config)?;
    Ok(())
}

fn read_config(input: &Path) -> Result<SiteConfig> {
    let config_path = input.join("mdsite.toml");
    if !config_path.exists() {
        return Err(crate::error::Error::Other(format!(
            "missing required site configuration: {}",
            config_path.display()
        )));
    }
    let config = fs::read_to_string(&config_path).map_err(|error| {
        crate::error::Error::Other(format!("read {}: {error}", config_path.display()))
    })?;
    let value: toml::Value = toml::from_str(&config).map_err(|error| {
        crate::error::Error::Other(format!("parse {}: {error}", config_path.display()))
    })?;
    let domain = value
        .get("default")
        .and_then(toml::Value::as_table)
        .and_then(|section| section.get("domain"))
        .and_then(toml::Value::as_str)
        .filter(|domain| !domain.trim().is_empty())
        .ok_or_else(|| {
            crate::error::Error::Other(format!(
                "{} must define a non-empty [default].domain",
                config_path.display()
            ))
        })?
        .trim()
        .trim_end_matches('/')
        .to_owned();
    let llms_prefix = value
        .get("llms")
        .and_then(toml::Value::as_table)
        .and_then(|section| section.get("prefix"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned);
    let default_section = value.get("default").and_then(toml::Value::as_table);
    let generate_sitemap = config_boolean(default_section, "generate_sitemap");
    let generate_llms = config_boolean(default_section, "generate_llms");
    let generate_rss = config_boolean(default_section, "generate_rss");
    let generate_robots = config_boolean(default_section, "generate_robots");

    let domain = if domain.starts_with("http://") || domain.starts_with("https://") {
        domain
    } else {
        format!("https://{domain}")
    };

    Ok(SiteConfig {
        domain,
        llms_prefix,
        generate_sitemap,
        generate_llms,
        generate_rss,
        generate_robots,
    })
}

fn config_boolean(section: Option<&toml::map::Map<String, toml::Value>>, key: &str) -> bool {
    section
        .and_then(|section| section.get(key))
        .and_then(toml::Value::as_bool)
        .unwrap_or(true)
}

fn generate_robots(input: &Path, output: &Path, config: &SiteConfig) -> Result<()> {
    if !config.generate_robots || input.join("robots.txt").is_file() {
        return Ok(());
    }

    let mut robots = String::from("User-agent: *\nAllow: /\n");
    if config.generate_sitemap {
        robots.push_str(&format!("\nSitemap: {}/sitemap.xml\n", config.domain));
    }
    fs::write(output.join("robots.txt"), robots)?;
    Ok(())
}

fn convert_file(file: &MdFile, output_root: &Path, include_rss_link: bool) -> Result<()> {
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
    let footer = parsed.footer.as_deref().and_then(footer_html);
    let page = html::render_page(
        &title,
        &body,
        &css_hrefs,
        Some(md_href),
        &file.relative,
        parsed.description.as_deref(),
        parsed.language.as_deref(),
        parsed.publish_date.as_ref(),
        parsed.last_updated_at.as_ref(),
        footer.as_deref(),
        include_rss_link.then(|| css::relative_asset_href(&html_rel, "rss.xml")),
    );

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

fn generate_directory_pages(
    files: &[MdFile],
    output_root: &Path,
    include_rss_link: bool,
) -> Result<Vec<PathBuf>> {
    let mut files_by_directory = BTreeMap::<_, Vec<_>>::new();
    let mut generated_pages = Vec::new();
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
        let page = html::render_page(
            &title,
            &body,
            &css_hrefs,
            md_href,
            &html_rel,
            None,
            None,
            None,
            None,
            None,
            include_rss_link.then(|| css::relative_asset_href(&html_rel, "rss.xml")),
        );
        fs::write(output_root.join(html_rel), page)?;
        fs::write(output_root.join(md_rel), markdown)?;
        generated_pages.push(directory.with_extension("md"));

        let source_index = directory.join("index.md");
        if !files.iter().any(|file| file.relative == source_index) {
            let legacy_index = output_root.join(&directory).join("index.html");
            if legacy_index.exists() {
                fs::remove_file(legacy_index)?;
            }
        }
    }

    Ok(generated_pages)
}

fn footer_html(markdown: &str) -> Option<String> {
    let html = markdown_to_html(markdown);
    html.strip_prefix("<p>")
        .and_then(|html| html.strip_suffix("</p>\n"))
        .map(str::to_owned)
}

fn generate_site_metadata(
    files: &[MdFile],
    directory_pages: &[PathBuf],
    output_root: &Path,
    config: &SiteConfig,
) -> Result<()> {
    let source_metadata = files
        .iter()
        .map(|file| Ok((file.relative.clone(), page_metadata(file)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let mut markdown_pages = files
        .iter()
        .map(|file| file.relative.clone())
        .collect::<Vec<_>>();
    markdown_pages.extend_from_slice(directory_pages);
    markdown_pages.sort();

    let date = generation_date()?;
    if config.generate_sitemap {
        let mut sitemap = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://sitemaps.org\">\n",
        );
        for markdown_page in &markdown_pages {
            let metadata = source_metadata.get(markdown_page);
            if metadata.is_some_and(|metadata| !metadata.include_in_sitemap) {
                continue;
            }
            let html_page = markdown_page.with_extension("html");
            let path = html_page.to_string_lossy().replace('\\', "/");
            let location = page_url(&config.domain, &path);
            let last_modified = metadata
                .and_then(|metadata| metadata.last_updated_at.as_ref())
                .or_else(|| metadata.and_then(|metadata| metadata.publish_date.as_ref()))
                .unwrap_or(&date);
            sitemap.push_str(&format!(
                "  <url>\n    <loc>{location}</loc>\n    <lastmod>{}</lastmod>\n  </url>\n",
                last_modified.to_rfc3339_opts(SecondsFormat::Secs, true),
            ));
        }
        sitemap.push_str("</urlset>\n");
        fs::write(output_root.join("sitemap.xml"), sitemap)?;
    }

    if config.generate_llms {
        let mut llms = String::from(
            "[Full site content](llms-full.txt): A full dump of every page on this site in one file.\n\n",
        );
        if let Some(prefix) = &config.llms_prefix {
            llms.push_str(prefix);
            if !prefix.ends_with('\n') {
                llms.push('\n');
            }
            llms.push('\n');
        }
        for markdown_page in &markdown_pages {
            let path = markdown_page.to_string_lossy().replace('\\', "/");
            llms.push_str(&format!("- [{path}]({path})\n"));
        }
        fs::write(output_root.join("llms.txt"), llms)?;

        let mut llms_full = String::new();
        for markdown_page in &markdown_pages {
            let markdown = fs::read_to_string(output_root.join(markdown_page))?;
            llms_full.push_str(&markdown);
            if !markdown.ends_with('\n') {
                llms_full.push('\n');
            }
            llms_full.push('\n');
        }
        fs::write(output_root.join("llms-full.txt"), llms_full)?;
    }

    if config.generate_rss {
        let home_metadata = source_metadata.get(Path::new("index.md"));
        let feed_title = home_metadata
            .and_then(|metadata| metadata.title.as_deref())
            .unwrap_or(&config.domain);
        let feed_description = home_metadata
            .and_then(|metadata| metadata.description.as_deref())
            .unwrap_or("RSS feed");
        let feed_language = home_metadata
            .and_then(|metadata| metadata.language.as_deref())
            .unwrap_or("en");
        let items = files
            .iter()
            .filter_map(|file| {
                let metadata = source_metadata.get(&file.relative)?;
                metadata.include_in_rss.then_some(rss::Item {
                    title: metadata.title.as_deref().unwrap_or_else(|| {
                        file.relative
                            .file_stem()
                            .and_then(|name| name.to_str())
                            .unwrap_or("page")
                    }),
                    description: metadata.description.as_deref(),
                    language: metadata.language.as_deref(),
                    link: page_url(
                        &config.domain,
                        &file
                            .relative
                            .with_extension("html")
                            .to_string_lossy()
                            .replace('\\', "/"),
                    ),
                    publish_date: metadata.publish_date.as_ref(),
                    last_updated_at: metadata.last_updated_at.as_ref(),
                })
            })
            .collect();
        fs::write(
            output_root.join("rss.xml"),
            rss::generate(rss::Feed {
                title: feed_title,
                description: feed_description,
                language: feed_language,
                link: &config.domain,
                build_date: &date,
                items,
            }),
        )?;
    }
    Ok(())
}

fn page_metadata(file: &MdFile) -> Result<PageMetadata> {
    let markdown = fs::read_to_string(&file.absolute)?;
    let parsed = frontmatter::parse(&markdown);
    Ok(PageMetadata {
        title: parsed.title,
        description: parsed.description,
        language: parsed.language,
        publish_date: parsed.publish_date,
        last_updated_at: parsed.last_updated_at,
        include_in_rss: parsed.include_in_rss,
        include_in_sitemap: parsed.include_in_sitemap,
    })
}

fn page_url(domain: &str, path: &str) -> String {
    if path == "index.html" {
        domain.to_owned()
    } else {
        format!("{domain}/{path}")
    }
}

fn generation_date() -> Result<DateTime<Utc>> {
    Ok(Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn write_config(input: &Path) {
        fs::write(
            input.join("mdsite.toml"),
            "[default]\ndomain = \"example.com\"\n",
        )
        .unwrap();
    }

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
        write_config(input.path());

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
        write_config(input.path());
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
        write_config(input.path());

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
    fn build_generates_sitemap_and_llms_files_from_configured_pages() {
        let input = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        fs::create_dir_all(input.path().join("guides")).unwrap();
        fs::write(
            input.path().join("index.md"),
            "---\ntitle: Home\ndescription: Home page\nlanguage: en\npublish_date: 2026-01-02T20:31\n---\nWelcome home.\n",
        )
        .unwrap();
        fs::write(
            input.path().join("guides/start.md"),
            "---\ntitle: Start\ndescription: Get started\nlanguage: nl\nlast_updated_at: 2026-03-04T20:31:00+02:00\ninclude_in_sitemap: false\n---\nGet started.\n",
        )
        .unwrap();
        fs::write(
            input.path().join("guides/hidden.md"),
            "---\ntitle: Hidden\npublish_date: 2026-02-03\ninclude_in_rss: false\n---\nHidden page.\n",
        )
        .unwrap();
        fs::write(
            input.path().join("mdsite.toml"),
            "[default]\ndomain = \"example.com/\"\n\n[llms]\nprefix = \"# Documentation\"\n",
        )
        .unwrap();

        build(input.path(), output.path()).unwrap();

        let date = generation_date().unwrap();
        let sitemap = fs::read_to_string(output.path().join("sitemap.xml")).unwrap();
        assert!(sitemap.contains("<loc>https://example.com</loc>"));
        assert!(sitemap.contains("<loc>https://example.com/guides.html</loc>"));
        assert!(sitemap.contains("<loc>https://example.com/guides/hidden.html</loc>"));
        assert!(!sitemap.contains("https://example.com/guides/start.html"));
        assert!(sitemap.contains("<lastmod>2026-01-02T20:31:00Z</lastmod>"));
        assert!(sitemap.contains("<lastmod>2026-02-03T00:00:00Z</lastmod>"));
        assert!(sitemap.contains(&format!(
            "<lastmod>{}</lastmod>",
            date.to_rfc3339_opts(SecondsFormat::Secs, true)
        )));

        let llms = fs::read_to_string(output.path().join("llms.txt")).unwrap();
        assert!(llms.starts_with("[Full site content](llms-full.txt):"));
        assert!(llms.contains("# Documentation"));
        assert!(llms.contains("- [guides.md](guides.md)"));
        assert!(llms.contains("- [guides/hidden.md](guides/hidden.md)"));
        assert!(llms.contains("- [guides/start.md](guides/start.md)"));
        assert!(llms.contains("- [index.md](index.md)"));

        let llms_full = fs::read_to_string(output.path().join("llms-full.txt")).unwrap();
        assert!(llms_full.contains("# Home\n\nWelcome home."));
        assert!(llms_full.contains("# guides\n\n- [hidden](guides/hidden.html)"));
        assert!(llms_full.contains("# Start\n\nGet started."));
        let home = fs::read_to_string(output.path().join("index.html")).unwrap();
        assert!(home.contains("<html lang=\"en\">"));
        assert!(home.contains("<meta name=\"description\" content=\"Home page\">"));
        assert!(
            home.contains("published <time datetime=\"2026-01-02T20:31:00Z\">02-01-2026</time>")
        );

        let rss = fs::read_to_string(output.path().join("rss.xml")).unwrap();
        assert!(rss.contains("<title>Home</title>"));
        assert!(rss.contains("<description>Home page</description>"));
        assert!(rss.contains("https://example.com/guides/start.html"));
        assert!(rss.contains("<pubDate>Wed, 04 Mar 2026 18:31:00 GMT</pubDate>"));
        assert!(!rss.contains("https://example.com/guides/hidden.html"));
        assert!(!output.path().join("mdsite.toml").exists());
    }

    #[test]
    fn build_respects_disabled_metadata_generators() {
        let input = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        fs::write(input.path().join("index.md"), "# Home\n").unwrap();
        fs::write(
            input.path().join("mdsite.toml"),
            "[default]\ndomain = \"example.com\"\ngenerate_sitemap = false\ngenerate_llms = false\ngenerate_rss = false\n",
        )
        .unwrap();

        build(input.path(), output.path()).unwrap();

        assert!(!output.path().join("sitemap.xml").exists());
        assert!(!output.path().join("llms.txt").exists());
        assert!(!output.path().join("llms-full.txt").exists());
        assert!(!output.path().join("rss.xml").exists());
        let index = fs::read_to_string(output.path().join("index.html")).unwrap();
        assert!(!index.contains("application/rss+xml"));
    }

    #[test]
    fn build_generates_default_robots_with_optional_sitemap() {
        let input = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        fs::write(input.path().join("index.md"), "# Home\n").unwrap();
        write_config(input.path());

        build(input.path(), output.path()).unwrap();
        assert_eq!(
            fs::read_to_string(output.path().join("robots.txt")).unwrap(),
            "User-agent: *\nAllow: /\n\nSitemap: https://example.com/sitemap.xml\n"
        );

        fs::write(
            input.path().join("mdsite.toml"),
            "[default]\ndomain = \"example.com\"\ngenerate_sitemap = false\n",
        )
        .unwrap();
        build(input.path(), output.path()).unwrap();
        assert_eq!(
            fs::read_to_string(output.path().join("robots.txt")).unwrap(),
            "User-agent: *\nAllow: /\n"
        );
    }

    #[test]
    fn build_preserves_input_robots_and_requires_configuration() {
        let input = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        fs::write(input.path().join("index.md"), "# Home\n").unwrap();
        fs::write(
            input.path().join("robots.txt"),
            "User-agent: Example\nDisallow: /\n",
        )
        .unwrap();

        let error = build(input.path(), output.path()).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("missing required site configuration")
        );

        write_config(input.path());
        build(input.path(), output.path()).unwrap();
        assert_eq!(
            fs::read_to_string(output.path().join("robots.txt")).unwrap(),
            "User-agent: Example\nDisallow: /\n"
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
            "---\nfooter: \"Made with *care*.\"\n---\n# Diagram\n\n```mermaid\nflowchart LR\n  A[Start] --> B[End]\n```\n",
        )
        .unwrap();
        write_config(input.path());

        build(input.path(), output.path()).unwrap();

        let index = fs::read_to_string(output.path().join("index.html")).unwrap();
        assert!(index.contains("<pre class=\"mermaid\">"));
        assert!(index.contains("Start"));
        assert!(index.contains("class=\"e\""));
        assert!(index.contains("<div class=\"page-footer\">Made with <em>care</em>.</div>"));
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
        write_config(input.path());

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
        write_config(input.path());
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
