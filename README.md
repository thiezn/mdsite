# mdsite

Minimal static site generator: Markdown in, clean HTML out.

## Features

- Walk an input directory of `.md` files (relative folder structure preserved)
- Emit matching `.html` pages with minimal semantic HTML
- Copy each `.md` next to its `.html`, with a `markdown` source link and breadcrumb navigation on every page
- Shared `style.css` at the output root (correct relative paths from nested pages)
- Generate `sitemap.xml`, `llms.txt`, and `llms-full.txt` at the output root
- Generate `rss.xml` from opted-in Markdown pages
- Basic Markdown: headings, paragraphs, emphasis, strong, links, lists, inline code, fenced code blocks
- Mermaid: fenced Mermaid code blocks are rendered as styled, accessible HTML with the built-in Rust renderer

## Install

```bash
cargo install --path .
```

## Usage

```bash
mdsite build --input ./content --output ./site
```

## Site Configuration

An optional `mdsite.toml` at the input root configures generated site metadata.
It is not copied into the output directory. Set `[default].domain` to generate
absolute sitemap URLs; a domain without a scheme uses `https://`. The optional
`[llms].prefix` is inserted above the Markdown page links in `llms.txt`.

```toml
[default]
domain = "example.com"

[llms]
prefix = "# Example documentation"
```

Every build writes `llms.txt` and `llms-full.txt`. A configured domain also
populates `sitemap.xml` with every generated HTML page and the appropriate page
date. `rss.xml` is generated on every build and uses the configured domain for
page URLs.

## Page Frontmatter

Pages may start with frontmatter. `description` is emitted as an HTML metadata
description and used in RSS; `language` sets the page HTML language and RSS item
language. `publish_date` and `last_updated_at` use `YYYY-MM-DD` and appear at
the bottom-right of the page. The sitemap uses `last_updated_at`, then
`publish_date`, then the build date. Both inclusion flags default to `true`.

```markdown
---
title: "Example page"
description: "A concise page summary"
language: en
publish_date: 2026-09-01
last_updated_at: 2026-09-04
include_in_rss: true
include_in_sitemap: true
---
```

Library API:

```rust
mdsite::build(input_dir, output_dir)?;
```

Demo content lives in `examples/demo/`.

## CI and GitHub Pages

Workflow [`.github/workflows/pages.yml`](.github/workflows/pages.yml):

1. `cargo test`
2. `cargo build --release`
3. Generate `examples/demo` into `_site`
4. Upload the site as a workflow artifact (every PR/push)
5. On `main` (or `workflow_dispatch`), deploy `_site` to GitHub Pages

One-time repo setup: **Settings → Pages → Build and deployment → Source: GitHub Actions**.

After the first successful deploy from `main`, the site is at `https://thiezn.github.io/mdsite/`.

## Development

```bash
cargo test
cargo build --release
./target/release/mdsite build --input examples/demo --output build/mdsite-demo
```

## Mermaid

Mermaid diagrams require no browser JavaScript, Node.js, or external CLI. The generator renders supported diagrams into styled HTML during the normal build, so the output is ready to host as static files.

The renderer is based on the [Grok Build Mermaid renderer](https://github.com/xai-org/grok-build/crates/codegen/xai-grok-markdown/src/mermaid.rs) and Simon Willison's [WebAssembly adaptation](https://github.com/simonw/tools/tree/main/grok-mermaid).
