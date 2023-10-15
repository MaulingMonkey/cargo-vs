#![deny(dead_code)] // dead code here is a forwards compatability hazard per https://github.com/MaulingMonkey/cargo-vs/issues/5

use serde::*;

use std::collections::HashSet;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};



pub(crate) type PackageId = String;

/// cargo metadata<br>
/// `{ ... }`
#[derive(Deserialize, Debug)]
pub(crate) struct Root {
    pub workspace_root: PathBuf,
    pub packages: Vec<PackageRef>,
    pub workspace_members: HashSet<PackageId>,
    // metadata
    // ...
}

/// cargo metadata<br>
/// `{ "packages": [ ... ] }`
#[derive(Deserialize, Debug)]
pub(crate) struct PackageRef {
    //pub manifest_path: PathBuf,
    pub id:         PackageId,
    //pub name:       String,
    //pub version:    String,
    pub targets:    Vec<PackageTarget>,
    // metadata
    // ...
}

/// cargo metadata<br>
/// `{ "packages": [ { "manifest_path": "..." } ] }`
#[derive(Deserialize, Debug)]
pub(crate) struct PackageTarget {
    pub kind:           Vec<String>, // "lib", "example" (, "bin"?)
    //pub crate_types:    Vec<String>, // "lib", "bin"
    pub name:           String,
    //pub src_path:       PathBuf,
    //pub edition:        String,
    //pub doctest:        bool,
    //pub test:           bool,
    // ...
}



impl Root {
    pub fn get() -> io::Result<Self> {
        let o = Command::new("cargo").args(&["metadata", "--all-features", "--format-version", "1"]).stderr(Stdio::inherit()).output()?;
        match o.status.code() {
            Some(0) => {},
            Some(n) => return Err(io::Error::new(io::ErrorKind::BrokenPipe, format!("`cargo metadata` failed (exit code {})", n))),
            None    => return Err(io::Error::new(io::ErrorKind::BrokenPipe, "`cargo metadata` failed (signal)")),
        }
        let stdout = String::from_utf8(o.stdout).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        serde_json::from_str(&stdout).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }
}
