# mdsite

Minimal static site generator: Markdown in, clean HTML out.

## Features

- Walk an input directory of `.md` files (relative folder structure preserved)
- Emit matching `.html` pages with minimal semantic HTML
- Copy each `.md` next to its `.html`, with a Markdown source link on every page
- Shared `style.css` at the output root (correct relative paths from nested pages)
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
