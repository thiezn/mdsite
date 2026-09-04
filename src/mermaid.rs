//! Mermaid diagram rendering via `mmdc` (mermaid-cli).

use crate::error::{Error, Result};
use crate::markdown::MermaidBlock;
use std::path::{Path, PathBuf};
use std::process::Command;

/// True if `mmdc` is available on PATH.
pub fn mmdc_available() -> bool {
    Command::new("mmdc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Render Mermaid blocks to SVG files beside the HTML page.
///
/// Files are named `{stem}-mermaid-{index}.svg` in `output_dir`.
pub fn render_blocks(
    blocks: &[MermaidBlock],
    output_dir: &Path,
    stem: &str,
) -> Result<Vec<PathBuf>> {
    if blocks.is_empty() {
        return Ok(Vec::new());
    }
    if !mmdc_available() {
        return Err(Error::MermaidCliMissing);
    }

    std::fs::create_dir_all(output_dir)?;
    let mut written = Vec::new();

    for block in blocks {
        let svg_name = format!("{}-mermaid-{}.svg", stem, block.index);
        let svg_path = output_dir.join(&svg_name);
        let mmd_path = output_dir.join(format!("{}-mermaid-{}.mmd", stem, block.index));
        std::fs::write(&mmd_path, &block.source)?;

        let output = Command::new("mmdc")
            .arg("-i")
            .arg(&mmd_path)
            .arg("-o")
            .arg(&svg_path)
            .output();

        let output = match output {
            Ok(o) => o,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let _ = std::fs::remove_file(&mmd_path);
                return Err(Error::MermaidCliMissing);
            }
            Err(e) => {
                let _ = std::fs::remove_file(&mmd_path);
                return Err(Error::Io(e));
            }
        };

        let _ = std::fs::remove_file(&mmd_path);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let message = if !stderr.is_empty() { stderr } else { stdout };
            return Err(Error::MermaidRender {
                path: svg_path,
                message,
            });
        }

        written.push(svg_path);
    }

    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_blocks_ok_without_mmdc() {
        let dir = tempfile::tempdir().unwrap();
        let out = render_blocks(&[], dir.path(), "page").unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn missing_mmdc_returns_clear_error() {
        if mmdc_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let blocks = vec![MermaidBlock {
            index: 1,
            source: "graph TD; A-->B;".into(),
        }];
        let err = render_blocks(&blocks, dir.path(), "page").unwrap_err();
        match err {
            Error::MermaidCliMissing => {}
            other => panic!("expected MermaidCliMissing, got {other:?}"),
        }
    }
}
