use std::fs::File;
use std::io::{Cursor, Seek, Write, BufWriter};
use std::path::Path;
use byteorder::{LittleEndian, WriteBytesExt};
use super::error::Result;
use super::define::*;
/// Write a STL file to a path.
pub fn write_file<P: AsRef<Path>>(path: P, meshs: &[Mesh], format: Format) -> Result<()> {
    let file = File::open(path)?;
    write(file, meshs, format)
}

/// Write a STL file from bytes.
pub fn write_bytes(meshs: &[Mesh], format: Format) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    write(Cursor::new(&mut buffer), meshs, format)?;
    Ok(buffer)
}

/// Write a STL file to a writer.
pub fn write<W: Write + Seek>(writer: W, meshs: &[Mesh], format: Format) -> Result<()> {
    let mut writer = BufWriter::new(writer);
    match format {
        Format::Ascii => write_ascii(&mut writer, meshs)?,
        Format::Binary => write_binary(&mut writer, &meshs[0])?,
    }
    Ok(())
}

/// Write a STL file to a writer in ASCII format.
pub fn write_ascii<W: Write + Seek>(writer: W, meshs: &[Mesh]) -> Result<()> {
    let mut writer = BufWriter::new(writer);
    for mesh in meshs {
        let name = mesh.name.as_deref().unwrap_or_default();
        writeln!(writer, "{}", ["solid", name].join(" "))?;
        for (i, v) in mesh.indices.iter().enumerate() {
            writeln!(writer, "  facet normal {:?}", mesh.normals[i])?;
            writeln!(writer, "    outer loop")?;
            writeln!(writer, "     vertex {:?}", mesh.vertices[v[0]])?;
            writeln!(writer, "     vertex {:?}", mesh.vertices[v[1]])?;
            writeln!(writer, "     vertex {:?}", mesh.vertices[v[2]])?;
            writeln!(writer, "    endloop")?;
            writeln!(writer, "  endfacet")?;
        }
        writeln!(writer, "{}", ["endsolid", name].join(" "))?;
    }
    Ok(())
}

pub fn write_binary<W: Write + Seek>(writer: W, mesh: &Mesh) -> Result<()> {
    let mut writer = BufWriter::new(writer);
    // write header
    writer.write_all(&mesh.header.unwrap_or([0u8; 80]))?;
    // write face count
    writer.write_u32::<LittleEndian>(mesh.indices.len() as u32)?;
    // write faces
    for (i, v) in mesh.indices.iter().enumerate() {
        let normal = mesh.normals[i];
        writer.write_f32::<LittleEndian>(normal.x)?;
        writer.write_f32::<LittleEndian>(normal.y)?;
        writer.write_f32::<LittleEndian>(normal.z)?;
        for i in v.iter() {
            let vertex = mesh.vertices[*i];
            writer.write_f32::<LittleEndian>(vertex.x)?;
            writer.write_f32::<LittleEndian>(vertex.y)?;
            writer.write_f32::<LittleEndian>(vertex.z)?;
        }
        let attr = mesh.attrs.as_ref()
            .map(|attrs| attrs[i])
            .unwrap_or(0);
        writer.write_u16::<LittleEndian>(attr)?;
    }
    Ok(())
}