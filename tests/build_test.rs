use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn mdsite_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mdsite"))
}

#[test]
fn cli_build_fixtures() {
    let input = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let output = tempfile::tempdir().unwrap();

    let status = Command::new(mdsite_bin())
        .args([
            "build",
            "--input",
            input.to_str().unwrap(),
            "--output",
            output.path().to_str().unwrap(),
        ])
        .status()
        .expect("run mdsite");
    assert!(status.success());

    let index = fs::read_to_string(output.path().join("index.html")).unwrap();
    assert!(index.contains("rel=\"alternate\" type=\"text/markdown\" href=\"index.md\""));
    assert!(index.contains("href=\"index.md\""));
    assert!(index.contains("href=\"style.css\""));
    assert!(output.path().join("index.md").is_file());
    assert!(output.path().join("style.css").is_file());

    let nested = fs::read_to_string(output.path().join("nested/guide.html")).unwrap();
    assert!(nested.contains("href=\"../style.css\""));
    assert!(nested.contains("href=\"guide.md\""));
    assert!(
        nested.contains("rel=\"alternate\" type=\"text/markdown\" href=\"../nested/guide.md\"")
    );
    assert!(nested.contains("href=\"../index.html\">home</a>"));
    assert!(nested.contains("href=\"../nested.html\">nested</a>"));
}
