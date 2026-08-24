use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use super::error::Result;
use super::define::package::*;

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

/// Write the package to a file.
pub fn write_file<P: AsRef<Path>>(package: &Package, path: P) -> Result<()> {
    let file = File::create(path)?;
    package.write(file)
}

/// Write the package to bytes.
pub fn write_bytes(package: &Package) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    package.write(Cursor::new(&mut buffer))?;
    Ok(buffer)
}