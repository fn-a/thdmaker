//! AMF File Writer
//! 
//! Provides the functionality to write AMF format data to files or memory, supporting:
//! - AMF files in XML format
//! - AMF files compressed with ZIP

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::{ZipWriter, CompressionMethod};
use quick_xml::Writer;
use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText, BytesDecl};
use super::error::Result;
use super::define::*;

impl Document {
    // Write AMF document to a generic writer (XML format)
    pub fn write<W: Write>(&self, writer: W) -> Result<()> {
        let mut xml_writer = Writer::new_with_indent(writer, b' ', 2);
        
        // Write XML declaration
        xml_writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
        
        // Write newline after declaration
        xml_writer.get_mut().write_all(b"\n")?;
        
        // Write AMF root element
        let mut amf_elem = BytesStart::new("amf");
        amf_elem.push_attribute(("xmlns", "http://amf.io/2011/amf"));
        
        if let Some(ref unit) = self.unit {
            amf_elem.push_attribute(("unit", unit.to_string().as_str()));
        }
        if let Some(version) = self.version {
            amf_elem.push_attribute(("version", version.to_string().as_str()));
        }
        if let Some(ref lang) = self.lang {
            amf_elem.push_attribute(("xml:lang", lang.as_str()));
        }
        
        xml_writer.write_event(Event::Start(amf_elem))?;
        
        // Write metadata
        for metadata in &self.metadatas {
            metadata.write(&mut xml_writer)?;
        }
        
        // Write objects
        for object in &self.objects {
            object.write(&mut xml_writer)?;
        }
        
        // Write materials
        for material in &self.materials {
            material.write(&mut xml_writer)?;
        }
        
        // Write textures
        for texture in &self.textures {
            texture.write(&mut xml_writer)?;
        }
        
        // Write constellations
        for constellation in &self.constellations {
            constellation.write(&mut xml_writer)?;
        }
        
        // Close AMF element
        xml_writer.write_event(Event::End(BytesEnd::new("amf")))?;

        Ok(())
    }
}

impl Metadata {
    // Write metadata element
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("metadata");
        elem.push_attribute(("type", self.r#type.as_str()));
        
        writer.write_event(Event::Start(elem))?;
        
        writer.write_event(Event::Text(BytesText::new(&self.value)))?;
        
        writer.write_event(Event::End(BytesEnd::new("metadata")))?;
        
        Ok(())
    }
}

impl Object {
    // Write object element
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("object");
        elem.push_attribute(("id", self.id.to_string().as_str()));
        
        writer.write_event(Event::Start(elem))?;
        
        // Write metadata
        for metadata in &self.metadatas {
            metadata.write(writer)?;
        }
        
        // Write colors
        for color in &self.colors {
            color.write(writer)?;
        }
        
        // Write mesh
        self.mesh.write(writer)?;
        
        writer.write_event(Event::End(BytesEnd::new("object")))?;
        
        Ok(())
    }
}

impl Mesh  {
    // Write mesh element
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("mesh")))?;
        
        // Write vertices
        self.vertices.write(writer)?;
        
        // Write volume
        self.volume.write(writer)?;
        
        writer.write_event(Event::End(BytesEnd::new("mesh")))?;
        
        Ok(())
    }
}

impl Vertices {
    // Write vertices element
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("vertices")))?;
        
        // Write vertices
        for vertex in &self.vertices {
            vertex.write(writer)?;
        }
        
        // Write edges
        for edge in &self.edges {
            edge.write(writer)?;
        }
        
        writer.write_event(Event::End(BytesEnd::new("vertices")))?;
        
        Ok(())
    }
}

impl Vertex {
    // Write vertex element
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("vertex")))?;
        
        // Write metadata
        for metadata in &self.metadatas {
            metadata.write(writer)?;
        }
        
        // Write coordinates
        self.coordinates.write(writer)?;
        
        // Write colors
        for color in &self.colors {
            color.write(writer)?;
        }
        
        // Write normals
        for normal in &self.normals {
            normal.write(writer)?;
        }
        
        writer.write_event(Event::End(BytesEnd::new("vertex")))?;
        
        Ok(())
    }
}

impl Coordinates {
    // Write coordinates element
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("coordinates")))?;
        
        NameValue::from(("x", self.x.to_string().as_str())).write(writer)?;
        NameValue::from(("y", self.y.to_string().as_str())).write(writer)?;
        NameValue::from(("z", self.z.to_string().as_str())).write(writer)?;
        
        writer.write_event(Event::End(BytesEnd::new("coordinates")))?;
        
        Ok(())
    }
}

impl Normal {
    // Write normal element
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("normal")))?;
        
        NameValue::from(("nx", self.nx.to_string().as_str())).write(writer)?;
        NameValue::from(("ny", self.ny.to_string().as_str())).write(writer)?;
        NameValue::from(("nz", self.nz.to_string().as_str())).write(writer)?;
        
        writer.write_event(Event::End(BytesEnd::new("normal")))?;
        
        Ok(())
    }
}

impl Edge {
    // Write edge element
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("edge")))?;
        
        NameValue::from(("v1", self.v1.to_string().as_str())).write(writer)?;
        NameValue::from(("dx1", self.dx1.to_string().as_str())).write(writer)?;
        NameValue::from(("dy1", self.dy1.to_string().as_str())).write(writer)?;
        NameValue::from(("dz1", self.dz1.to_string().as_str())).write(writer)?;
        NameValue::from(("v2", self.v2.to_string().as_str())).write(writer)?;
        NameValue::from(("dx2", self.dx2.to_string().as_str())).write(writer)?;
        NameValue::from(("dy2", self.dy2.to_string().as_str())).write(writer)?;
        NameValue::from(("dz2", self.dz2.to_string().as_str())).write(writer)?;
        
        writer.write_event(Event::End(BytesEnd::new("edge")))?;
        
        Ok(())
    }
}

impl Volume {
    // Write volume element
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("volume");
        
        if let Some(material_id) = self.material_id {
            elem.push_attribute(("materialid", material_id.to_string().as_str()));
        }
        if let Some(ref vtype) = self.r#type {
            elem.push_attribute(("type", vtype.to_string().as_str()));
        }
        
        writer.write_event(Event::Start(elem))?;
        
        // Write metadata
        for metadata in &self.metadatas {
            metadata.write(writer)?;
        }
        
        // Write colors
        for color in &self.colors {
            color.write(writer)?;
        }
        
        // Write triangles
        for triangle in &self.triangles {
            triangle.write(writer)?;
        }
        
        writer.write_event(Event::End(BytesEnd::new("volume")))?;
        
        Ok(())
    }
}

impl Triangle {
    // Write triangle element
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("triangle")))?;
        
        NameValue::from(("v1", self.v1.to_string().as_str())).write(writer)?;
        NameValue::from(("v2", self.v2.to_string().as_str())).write(writer)?;
        NameValue::from(("v3", self.v3.to_string().as_str())).write(writer)?;
        
        // Write texmap if present
        if let Some(ref texmap) = self.texmap {
            texmap.write(writer)?;
        }
        
        // Write colors
        for color in &self.colors {
            color.write(writer)?;
        }
        
        writer.write_event(Event::End(BytesEnd::new("triangle")))?;
        
        Ok(())
    }
}
impl TexMap {
    // Write texmap element
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("texmap");
        
        if let Some(rtexid) = self.rtexid {
            elem.push_attribute(("rtexid", rtexid.to_string().as_str()));
        }
        if let Some(gtexid) = self.gtexid {
            elem.push_attribute(("gtexid", gtexid.to_string().as_str()));
        }
        if let Some(btexid) = self.btexid {
            elem.push_attribute(("btexid", btexid.to_string().as_str()));
        }
        if let Some(atexid) = self.atexid {
            elem.push_attribute(("atexid", atexid.to_string().as_str()));
        }
        
        writer.write_event(Event::Start(elem))?;
        
        NameValue::from(("utex1", self.utex1.to_string().as_str())).write(writer)?;
        NameValue::from(("utex2", self.utex2.to_string().as_str())).write(writer)?;
        NameValue::from(("utex3", self.utex3.to_string().as_str())).write(writer)?;
        NameValue::from(("vtex1", self.vtex1.to_string().as_str())).write(writer)?;
        NameValue::from(("vtex2", self.vtex2.to_string().as_str())).write(writer)?;
        NameValue::from(("vtex3", self.vtex3.to_string().as_str())).write(writer)?;
        
        if let Some(wtex1) = self.wtex1 {
            NameValue::from(("wtex1", wtex1.to_string().as_str())).write(writer)?;
        }
        if let Some(wtex2) = self.wtex2 {
            NameValue::from(("wtex2", wtex2.to_string().as_str())).write(writer)?;
        }
        if let Some(wtex3) = self.wtex3 {
            NameValue::from(("wtex3", wtex3.to_string().as_str())).write(writer)?;
        }
        
        writer.write_event(Event::End(BytesEnd::new("texmap")))?;
        
        Ok(())
    }
}

impl Color {
    // Write color element
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("color")))?;
        
        NameValue::from(("r", self.r.to_string().as_str())).write(writer)?;
        NameValue::from(("g", self.g.to_string().as_str())).write(writer)?;
        NameValue::from(("b", self.b.to_string().as_str())).write(writer)?;
        
        if let Some(ref a) = self.a {
            NameValue::from(("a", a.to_string().as_str())).write(writer)?;
        }
        
        writer.write_event(Event::End(BytesEnd::new("color")))?;
        
        Ok(())
    }
}

impl Material {
    // Write material element
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("material");
        elem.push_attribute(("id", self.id.to_string().as_str()));
        
        writer.write_event(Event::Start(elem))?;
        
        // Write metadata
        for metadata in &self.metadatas {
            metadata.write(writer)?;
        }
        
        // Write composites
        for composite in &self.composites {
            composite.write(writer)?;
        }
        
        writer.write_event(Event::End(BytesEnd::new("material")))?;
        
        Ok(())
    }
}
impl Composite {
    // Write composite element
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("composite");
        elem.push_attribute(("materialid", self.material_id.to_string().as_str()));
        
        writer.write_event(Event::Start(elem))?;
        
        writer.write_event(Event::Text(BytesText::new(&self.value)))?;
        
        writer.write_event(Event::End(BytesEnd::new("composite")))?;
        
        Ok(())
    }
}

impl Texture {
    // Write texture element
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("texture");
        elem.push_attribute(("id", self.id.to_string().as_str()));
        elem.push_attribute(("width", self.width.to_string().as_str()));
        elem.push_attribute(("height", self.height.to_string().as_str()));
        elem.push_attribute(("depth", self.depth.to_string().as_str()));
        elem.push_attribute(("tiled", if self.tiled { "true" } else { "false" }));
        elem.push_attribute(("type", self.r#type.to_string().as_str()));
        
        writer.write_event(Event::Start(elem))?;
        
        writer.write_event(Event::Text(BytesText::new(&self.value)))?;
        
        writer.write_event(Event::End(BytesEnd::new("texture")))?;
        
        Ok(())
    }
}

impl Constellation {
    // Write constellation element
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("constellation");
        elem.push_attribute(("id", self.id.to_string().as_str()));
        
        writer.write_event(Event::Start(elem))?;
        
        // Write instances
        for instance in &self.instances {
            instance.write(writer)?;
        }
        
        writer.write_event(Event::End(BytesEnd::new("constellation")))?;
        
        Ok(())
    }
}

impl Instance {
    // Write instance element
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("instance");
        elem.push_attribute(("objectid", self.object_id.to_string().as_str()));
        
        writer.write_event(Event::Start(elem))?;
        
        if let Some(deltax) = self.deltax {
            NameValue::from(("deltax", deltax.to_string().as_str())).write(writer)?;
        }
        if let Some(deltay) = self.deltay {
            NameValue::from(("deltay", deltay.to_string().as_str())).write(writer)?;
        }
        if let Some(deltaz) = self.deltaz {
            NameValue::from(("deltaz", deltaz.to_string().as_str())).write(writer)?;
        }
        if let Some(rx) = self.rx {
            NameValue::from(("rx", rx.to_string().as_str())).write(writer)?;
        }
        if let Some(ry) = self.ry {
            NameValue::from(("ry", ry.to_string().as_str())).write(writer)?;
        }
        if let Some(rz) = self.rz {
            NameValue::from(("rz", rz.to_string().as_str())).write(writer)?;
        }
        
        writer.write_event(Event::End(BytesEnd::new("instance")))?;
        
        Ok(())
    }
}

struct NameValue<'a>(&'a str, &'a str);

impl<'a> NameValue<'a> {
    // Helper function to write a simple element with text content
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new(self.0)))?;
        
        writer.write_event(Event::Text(BytesText::new(self.1)))?;
        
        writer.write_event(Event::End(BytesEnd::new(self.0)))?;
        
        Ok(())
    }
}

impl<'a> From<(&'a str, &'a str)> for NameValue<'a> {
    fn from(value: (&'a str, &'a str)) -> Self {
        Self(value.0, value.1)
    }
}

/// Write AMF document to XML file
pub fn write_xml_file<P: AsRef<Path>>(doc: &Document, path: P) -> Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    doc.write(writer)
}

/// Write AMF document to ZIP file
pub fn write_zip_file<P: AsRef<Path>>(doc: &Document, path: P) -> Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let mut zip = ZipWriter::new(writer);
    
    let mut buffer = Vec::new();
    // Write the AMF content to the ZIP archive
    doc.write(&mut buffer)?;

    let xml_content = String::from_utf8(buffer)?;
    
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);
    zip.start_file("model.amf", options)?;
    zip.write_all(xml_content.as_bytes())?;
    
    zip.finish()?;
    
    Ok(())
}

/// Write AMF document to bytes (XML format)
pub fn write_xml_bytes(doc: &Document) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    doc.write(&mut buffer)?;
    Ok(buffer)
}

/// Write AMF document to ZIP bytes
pub fn write_zip_bytes(doc: &Document) -> Result<Vec<u8>> {
    let buffer = Vec::new();
    let cursor = std::io::Cursor::new(buffer);
    let mut zip = ZipWriter::new(cursor);
    let mut buffer = Vec::new();
    doc.write(&mut buffer)?;
    let xml_content = String::from_utf8(buffer)?;
    
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);
    
    zip.start_file("model.amf", options)?;
    zip.write_all(xml_content.as_bytes())?;
    
    let cursor = zip.finish()?;
    
    Ok(cursor.into_inner())
}