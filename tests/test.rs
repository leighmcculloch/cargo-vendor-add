use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use flate2::{write::GzEncoder, Compression};
use std::{io::Cursor, process::Command};

#[test]
fn test() -> Result<(), Box<dyn std::error::Error>> {
    let vendor_dir = assert_fs::TempDir::new()?;

    let crate_dir = assert_fs::TempDir::new()?;
    crate_dir.child("Cargo.toml").write_str(
        r#"[package]
name = "my"
"#,
    )?;

    let crate_file = assert_fs::NamedTempFile::new("my.crate")?;
    let mut buf = vec![];
    {
        let mut tar = tar::Builder::new(GzEncoder::new(
            Cursor::new(&mut buf),
            Compression::default(),
        ));
        tar.append_dir_all("my-0.0.1", crate_dir.path())?;
        tar.finish()?;
    }
    crate_file.write_binary(buf.as_slice())?;

    let mut cmd = Command::cargo_bin("cargo-vendor-add")?;
    cmd.arg("vendor-add");
    cmd.arg("--crate").arg(crate_file.path());
    cmd.arg("--vendor-path").arg(vendor_dir.path());
    cmd.assert().success().stderr(format!(
        r#"reading: {0}
writing: {1}/my-0.0.1/
writing: {1}/my-0.0.1/.cargo-checksum.json (generated)
writing: {1}/my-0.0.1/Cargo.toml
"#,
        crate_file.path().to_string_lossy(),
        vendor_dir.path().to_string_lossy(),
    ));

    vendor_dir.child("my-0.0.1").child("Cargo.toml").assert(
        r#"[package]
name = "my"
"#,
    );
    vendor_dir
        .child("my-0.0.1")
        .child(".cargo-checksum.json")
        .assert(r#"{"files":{},"package":""}"#);

    Ok(())
}
