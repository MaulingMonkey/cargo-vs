use serde::*;

use std::collections::HashSet;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};



pub(crate) type Url = String;
pub(crate) type PackageId = String;

/// cargo metadata<br>
/// `{ ... }`
#[derive(Deserialize, Debug)]
pub(crate) struct Root {
    #[serde(rename="workspace_root")]
    pub workspace: WorkspaceRoot,
    pub packages: Vec<PackageRef>,
    pub workspace_members: HashSet<PackageId>,
    // metadata
    // ...
}

/// cargo metadata<br>
/// `{ "workspace_root": "..." }`
#[derive(Debug)]
pub(crate) struct WorkspaceRoot {
    pub dir:    PathBuf,
    pub toml:   Option<WorkspaceToml>,
}

/// cargo metadata<br>
/// `{ "packages": [ ... ] }`
#[derive(Deserialize, Debug)]
pub(crate) struct PackageRef {
    #[serde(rename="manifest_path")]
    pub manifest:   ManifestRef,
    pub id:         PackageId,
    pub name:       String,
    pub version:    String,
    pub targets:    Vec<PackageTarget>,
    // metadata
    // ...
}

/// cargo metadata<br>
/// `{ "packages": [ { "manifest_path": "..." } ] }`
#[derive(Deserialize, Debug)]
pub(crate) struct PackageTarget {
    pub kind:           Vec<String>, // "lib", "example" (, "bin"?)
    pub crate_types:    Vec<String>, // "lib", "bin"
    pub name:           String,
    //pub src_path:       PathBuf,
    //pub edition:        String,
    //pub doctest:        bool,
    //pub test:           bool,
}



/// cargo metadata<br>
/// `{ "packages": [ { "manifest_path": "..." } ] }`
#[derive(Debug)]
pub(crate) struct ManifestRef {
    pub path:   PathBuf,
    pub toml:   Manifest,
}



/// Cargo.toml<br>
/// (root - might contains `[workspace]`)
#[derive(Deserialize, Debug)]
pub(crate) struct WorkspaceToml {
    #[serde(default)]
    pub metadata: Metadata,
}

/// Cargo.toml<br>
/// (root - expected to contain `[package]`)
#[derive(Deserialize, Debug)]
pub(crate) struct Manifest {
    pub package: Package,
    #[serde(default)]
    pub metadata: Metadata,
    // ...
}


/// Cargo.toml<br>
/// `[package]`
#[derive(Deserialize, Debug)]
pub(crate) struct Package {
    pub name:           String,
    pub version:        String,
    pub repository:     Option<Url>,
    pub documentation:  Option<Url>,
    pub homepage:       Option<Url>,
    #[serde(default)]
    pub metadata:       Metadata,
    // ...
}

/// Cargo.toml<br>
/// `[package.metadata]` or<br>
/// `[workspace.metadata]`
#[derive(Deserialize, Debug, Default)]
pub(crate) struct Metadata {
    pub local_install: Option<de::IgnoredAny>,
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

impl<'de> Deserialize<'de> for WorkspaceRoot {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let dir = PathBuf::deserialize(d)?;
        let file = dir.join("Cargo.toml");
        let text = match std::fs::read_to_string(&file) {
            Ok(text) => text,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Self { dir, toml: None }),
            Err(err) => return Err(de::Error::custom(&format!("unable to read {}: {}", file.display(), err))),
        };
        Ok(Self {
            toml: Some(toml::from_str(&text).map_err(|err| de::Error::custom(&format!("unable to parse {}: {}", file.display(), err)))?),
            dir,
        })
    }
}

impl<'de> Deserialize<'de> for ManifestRef {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let path = PathBuf::deserialize(d)?;
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(err) => return Err(de::Error::custom(&format!("unable to read {}: {}", path.display(), err))),
        };
        Ok(Self {
            toml: toml::from_str(&text).map_err(|err| de::Error::custom(&format!("unable to parse {}: {}", path.display(), err)))?,
            path,
        })
    }
}
