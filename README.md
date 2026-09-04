# mdsite

Minimal static site generator: Markdown in, clean HTML out.

## Features

- Walk an input directory of `.md` files (relative folder structure preserved)
- Emit matching `.html` pages with minimal semantic HTML
- Copy each `.md` next to its `.html`, with a Markdown source link on every page
- Shared `style.css` at the output root (correct relative paths from nested pages)
- Basic Markdown: headings, paragraphs, emphasis, strong, links, lists, inline code, fenced code blocks
- Mermaid: fenced mermaid code blocks are rendered to SVG via mermaid-cli (`mmdc`)

## Install

```bash
cargo install --path .
```

## Usage

```bash
mdsite build --input ./content --output ./site
```

Library API:

```rust
mdsite::build(input_dir, output_dir)?;
```

Demo content lives in `examples/demo/`.

## Mermaid dependency

Pages that contain mermaid fences require mermaid-cli on your PATH.

```bash
npm install -g @mermaid-js/mermaid-cli
```

If `mmdc` is missing, mdsite returns `Error::MermaidCliMissing`. Diagrams are written beside the HTML as `{stem}-mermaid-{n}.svg`. The copied `.md` keeps the original fence.

## CI and GitHub Pages

Workflow [`.github/workflows/pages.yml`](.github/workflows/pages.yml):

1. `cargo test`
2. `cargo build --release`
3. Install `mmdc`, generate `examples/demo` into `_site`
4. Upload the site as a workflow artifact (every PR/push)
5. On `main` (or `workflow_dispatch`), deploy `_site` to GitHub Pages

One-time repo setup: **Settings → Pages → Build and deployment → Source: GitHub Actions**.

After the first successful deploy from `main`, the site is at `https://thiezn.github.io/mdsite/`.

## Development

```bash
cargo test
cargo build --release
./target/release/mdsite build --input examples/demo --output /tmp/mdsite-demo
```

Mermaid render tests skip when `mmdc` is absent.
