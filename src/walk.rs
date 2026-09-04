//! Walk an input directory for Markdown and asset files.

use crate::error::{Error, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A Markdown source file and its path relative to the input root.
#[derive(Debug, Clone)]
pub struct MdFile {
    pub absolute: PathBuf,
    /// Relative path including the `.md` filename (e.g. `docs/intro.md`).
    pub relative: PathBuf,
}

/// A non-Markdown source file and its path relative to the input root.
#[derive(Debug, Clone)]
pub struct AssetFile {
    pub absolute: PathBuf,
    /// Relative path including the filename (e.g. `images/logo.png`).
    pub relative: PathBuf,
}

/// Collect all `.md` files under `input`, preserving relative paths.
pub fn collect_markdown_files(input: &Path) -> Result<Vec<MdFile>> {
    if !input.is_dir() {
        return Err(Error::InputNotDirectory(input.to_path_buf()));
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(input).follow_links(false) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let is_md = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("md"))
            .unwrap_or(false);
        if !is_md {
            continue;
        }
        let relative = path
            .strip_prefix(input)
            .map_err(|e| Error::Other(format!("strip prefix: {e}")))?
            .to_path_buf();
        files.push(MdFile {
            absolute: path.to_path_buf(),
            relative,
        });
    }
    files.sort_by(|a, b| a.relative.cmp(&b.relative));
    Ok(files)
}

/// Collect all non-Markdown files under `input`, preserving relative paths.
pub fn collect_asset_files(input: &Path) -> Result<Vec<AssetFile>> {
    if !input.is_dir() {
        return Err(Error::InputNotDirectory(input.to_path_buf()));
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(input).follow_links(false) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let is_md = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("md"));
        if is_md {
            continue;
        }
        let relative = path
            .strip_prefix(input)
            .map_err(|error| Error::Other(format!("strip prefix: {error}")))?
            .to_path_buf();
        files.push(AssetFile {
            absolute: path.to_path_buf(),
            relative,
        });
    }
    files.sort_by(|left, right| left.relative.cmp(&right.relative));
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn collects_nested_md() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("nested")).unwrap();
        fs::write(dir.path().join("a.md"), "# A").unwrap();
        fs::write(dir.path().join("nested/b.md"), "# B").unwrap();
        fs::write(dir.path().join("skip.txt"), "nope").unwrap();

        let files = collect_markdown_files(dir.path()).unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].relative, PathBuf::from("a.md"));
        assert_eq!(files[1].relative, PathBuf::from("nested/b.md"));
    }

    #[test]
    fn collects_nested_non_markdown_files() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("images")).unwrap();
        fs::write(dir.path().join("index.md"), "# Home").unwrap();
        fs::write(dir.path().join("site.webmanifest"), "{}").unwrap();
        fs::write(dir.path().join("images/logo.png"), [0, 1, 2]).unwrap();

        let files = collect_asset_files(dir.path()).unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].relative, PathBuf::from("images/logo.png"));
        assert_eq!(files[1].relative, PathBuf::from("site.webmanifest"));
    }
}
