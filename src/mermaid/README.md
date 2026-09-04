# Mermaid Renderer

This code is taken from Simon Willis port of the Grok Build agent mermaid renderer.


https://github.com/simonw/tools/tree/main/grok-mermaid


Grok build released a really nice ASCII renderer in a single file. Simon created a Rust crate that compiles this into web assembly so it can be nicely rendered on a website. The Grok file is here: xai-org/grok-build/crates/codegen/xai-grok-markdown/src/mermaid.rs. I've renamed it to renderer.rs.

As of now there doesn't seem to be any changed to those repos but good to periodically check the grok build one for improvements. Simon's tool was very lightweight so we've copied the relevant code parts here, but will iterate on it ourselves if needed to streamline it into this project.

