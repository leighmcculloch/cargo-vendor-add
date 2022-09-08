//! Add crates directly to a cargo vendor directly.
//!
//! # Install
//!
//! ```
//! cargo install --locked cargo-vendor-add
//! ```
//!
//! # Usage
//!
//! Add a `.crate` file to a `vendor/` directory.
//!
//! ```
//! cargo vendor-add --crate my.crate --vendor-path vendor/
//! ```

#![allow(clippy::missing_errors_doc)]

use clap::{AppSettings, Parser};
use flate2::read::GzDecoder;
use std::{
    fs::{self, File},
    io::{self, Write},
};
use tar::Archive;

#[derive(Parser, Debug)]
#[clap(
    version,
    about,
    disable_help_subcommand = true,
    disable_version_flag = true
)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
#[clap(bin_name = "cargo")]
enum RootCmd {
    VendorAdd(VendorAddCmd),
}

#[derive(Parser, Debug)]
#[clap(version, about)]
struct VendorAddCmd {
    /// Crate file path
    #[clap(long, parse(from_os_str))]
    crate_: std::path::PathBuf,
    /// Vendor directory path
    #[clap(long, parse(from_os_str))]
    vendor_path: std::path::PathBuf,
}

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("opening crate file: {0}")]
    OpeningCrate(io::Error),
    #[error("reading crate: {0}")]
    ReadingCrate(io::Error),
    #[error("writing crate to the vendor-path: {0}")]
    WritingToVendor(io::Error),
    #[error("opening vendor path checksum file: {0}")]
    OpeningChecksumFile(io::Error),
    #[error("writing vendor path checksum file: {0}")]
    WritingChecksumFile(io::Error),
}

impl VendorAddCmd {
    pub fn run(&self) -> Result<(), Error> {
        eprintln!("reading: {}", self.crate_.to_string_lossy());
        let crate_ = File::open(&self.crate_).map_err(Error::OpeningCrate)?;
        let mut archive = Archive::new(GzDecoder::new(crate_));
        archive
            .entries()
            .map_err(Error::ReadingCrate)?
            .filter_map(Result::ok)
            .map(|entry| {
                let mut entry = entry;
                let path = entry.path().unwrap();
                if path.ends_with("Cargo.toml") {
                    let checksum_path = self
                        .vendor_path
                        .join(path.parent().unwrap().join(".cargo-checksum.json"));
                    eprintln!("writing: {} (generated)", checksum_path.to_string_lossy());
                    fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .open(checksum_path)
                        .map_err(Error::OpeningChecksumFile)?
                        // TODO: Generate actual checksum file.
                        .write_all(r#"{"files":{},"package":""}"#.as_bytes())
                        .map_err(Error::WritingChecksumFile)?;
                }
                eprintln!("writing: {}", self.vendor_path.join(path).to_string_lossy());
                entry
                    .unpack_in(&self.vendor_path)
                    .map_err(Error::WritingToVendor)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(())
    }
}

fn main() {
    if let Err(e) = match RootCmd::parse() {
        RootCmd::VendorAdd(cmd) => cmd.run(),
    } {
        eprintln!("error: {}", e);
    }
}
