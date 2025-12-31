use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use super::error::Result;
use super::define::package::Package;

pub use super::path;
pub use super::error;
pub use super::define;

mod package;
mod model;
mod primitive;

mod trianglesets;
mod beamlattice;
mod booleanoperations;
mod displacement;
mod material;
mod production;
mod securecontent;
mod slice;
mod volumetric;
mod implicit;

/// Read a 3MF package from a file.
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<Package> {
    let file = File::open(path)?;
    Package::read(file)
}

/// Read a 3MF package from bytes.
pub fn read_bytes(data: &[u8]) -> Result<Package> {
    let cursor = Cursor::new(data);
    Package::read(cursor)
}