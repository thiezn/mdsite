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

    cargo install --path .

## Usage

    mdsite build --input ./content --output ./site

Library API:

    mdsite::build(input_dir, output_dir)?;

## Mermaid dependency

Pages that contain mermaid fences require mermaid-cli on your PATH.
Install the npm package for mermaid-cli so mmdc is available.
If mmdc is missing, mdsite returns a clear error. Diagrams are written
beside the HTML as {stem}-mermaid-{n}.svg. The copied .md keeps the original fence.

## Development

    cargo test
    cargo build --release

Mermaid render tests skip when mmdc is absent.
