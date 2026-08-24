//! AMF File Reader
//! 
//! Provides the functionality to read AMF format data from files or memory, supporting:
//! - AMF files in XML format
//! - AMF files compressed with ZIP

use std::str;
use std::fs::File;
use std::io::{BufReader, BufRead, Read, Cursor, Seek};
use std::path::Path;
use zip::ZipArchive;
use quick_xml::events::{Event, BytesStart};
use quick_xml::Reader;
use quick_xml::escape::unescape;
use super::error::{Error, Result};
use super::define::*;

impl Document {
    // Parse AMF content (children of amf element)
    fn parse<R: BufRead>(self: &mut Self, reader: &mut Reader<R>) -> Result<()> {
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"object" => {
                            let object = Object::parse(reader, e)?;
                            self.objects.push(object);
                        }
                        b"material" => {
                            let material = Material::parse(reader, e)?;
                            self.materials.push(material);
                        }
                        b"texture" => {
                            let texture = Texture::parse(reader, e)?;
                            self.textures.push(texture);
                        }
                        b"constellation" => {
                            let constellation = Constellation::parse(reader, e)?;
                            self.constellations.push(constellation);
                        }
                        b"metadata" => {
                            let metadata = Metadata::parse(reader, e)?;
                            self.metadatas.push(metadata);
                        }
                        _ => {},
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"amf" => break,
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(())
    }

    // Parse AMF root element attributes
    fn attributes(self: &mut Self, elem: &BytesStart) -> Result<()> {
        for attr in elem.attributes().flatten() {
            match attr.key.as_ref() {
                b"unit" => {
                    let value = attr.unescape_value()?;
                    self.unit = Some(Unit::from(value.as_ref()));
                }
                b"version" => {
                    let value = attr.unescape_value()?;
                    self.version = Some(value.parse()?);
                }
                b"xml:lang" | b"lang" => {
                    let value = attr.unescape_value()?;
                    self.lang = Some(value.to_string());
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl Object {
    // Parse object element
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut object = Self::new(0);
        
        // Parse attributes
        for attr in elem.attributes().flatten() {
            if attr.key.as_ref() == b"id" {
                let value = attr.unescape_value()?;
                object.id = value.parse().unwrap_or(0);
            }
        }
        
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"mesh" => {
                            object.mesh = Mesh::parse(reader)?;
                        }
                        b"metadata" => {
                            let metadata = Metadata::parse(reader, e)?;
                            object.metadatas.push(metadata);
                        }
                        b"color" => {
                            let color = Color::parse(reader)?;
                            object.colors.push(color);
                        }
                        _ => {},
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"object" => break,
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(object)
    }
}

impl Mesh {
    // Parse mesh element
    fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut mesh = Self::new();
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"vertices" => {
                            mesh.vertices = Vertices::parse(reader)?;
                        }
                        b"volume" => {
                            mesh.volume = Volume::parse(reader, e)?;
                        }
                        _ => {},
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"mesh" => break,
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(mesh)
    }
}

impl Vertices {
    // Parse vertices element
    fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut vertices = Self::default();
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"vertex" => {
                            let vertex = Vertex::parse(reader)?;
                            vertices.vertices.push(vertex);
                        }
                        b"edge" => {
                            let edge = Edge::parse(reader)?;
                            vertices.edges.push(edge);
                        }
                        _ => {},
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"vertices" => break,
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(vertices)
    }
}

impl Vertex {
    // Parse vertex element
    fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut vertex = Self::default();
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"coordinates" => {
                            vertex.coordinates = Coordinates::parse(reader)?;
                        }
                        b"metadata" => {
                            let metadata = Metadata::parse(reader, e)?;
                            vertex.metadatas.push(metadata);
                        }
                        b"color" => {
                            let color = Color::parse(reader)?;
                            vertex.colors.push(color);
                        }
                        b"normal" => {
                            let normal = Normal::parse(reader)?;
                            vertex.normals.push(normal);
                        }
                        _ => {},
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"vertex" => break,
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(vertex)
    }
}

impl Coordinates {
    // Parse coordinates element
    fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut coords = Self::default();
        let mut buf = Vec::new();
        let mut current: Option<String> = None;
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    current = Some(String::from_utf8_lossy(e.name().as_ref()).to_string());
                }
                Ok(Event::Text(ref e)) => {
                    if let Some(ref tag) = current {
                        let content = e.decode()?;
                        let text = unescape(&content)?.to_string();
                        let value: f64 = text.trim().parse().unwrap_or(0.0);
                        match tag.as_str() {
                            "x" => coords.x = value,
                            "y" => coords.y = value,
                            "z" => coords.z = value,
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if e.name().as_ref() == b"coordinates" {
                        break;
                    }
                    current = None;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(coords)
    }
}

impl Normal {
    // Parse normal element
    fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut normal = Self::default();
        let mut buf = Vec::new();
        let mut current: Option<String> = None;
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    current = Some(String::from_utf8_lossy(e.name().as_ref()).to_string());
                }
                Ok(Event::Text(ref e)) => {
                    if let Some(ref tag) = current {
                        let content = e.decode()?;
                        let text = unescape(&content)?.to_string();
                        let value: f64 = text.trim().parse().unwrap_or(0.0);
                        match tag.as_str() {
                            "nx" => normal.nx = value,
                            "ny" => normal.ny = value,
                            "nz" => normal.nz = value,
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if e.name().as_ref() == b"normal" {
                        break;
                    }
                    current = None;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(normal)
    }
}

impl Edge {
    // Parse edge element
    fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut edge = Self::default();
        let mut buf = Vec::new();
        let mut current: Option<String> = None;
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    current = Some(String::from_utf8_lossy(e.name().as_ref()).to_string());
                }
                Ok(Event::Text(ref e)) => {
                    if let Some(ref tag) = current {
                        let content = e.decode()?;
                        let text = unescape(&content)?.to_string();
                        match tag.as_str() {
                            "v1" => edge.v1 = text.trim().parse().unwrap_or(0),
                            "dx1" => edge.dx1 = text.trim().parse().unwrap_or(0.0),
                            "dy1" => edge.dy1 = text.trim().parse().unwrap_or(0.0),
                            "dz1" => edge.dz1 = text.trim().parse().unwrap_or(0.0),
                            "v2" => edge.v2 = text.trim().parse().unwrap_or(0),
                            "dx2" => edge.dx2 = text.trim().parse().unwrap_or(0.0),
                            "dy2" => edge.dy2 = text.trim().parse().unwrap_or(0.0),
                            "dz2" => edge.dz2 = text.trim().parse().unwrap_or(0.0),
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if e.name().as_ref() == b"edge" {
                        break;
                    }
                    current = None;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(edge)
    }
}

impl Volume {
    // Parse volume element
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut volume = Self::default();
        
        // Parse attributes
        for attr in elem.attributes().flatten() {
            match attr.key.as_ref() {
                b"materialid" => {
                    let value = attr.unescape_value()?;
                    volume.material_id = Some(value.parse().unwrap_or(0));
                }
                b"type" => {
                    let value = attr.unescape_value()?;
                    volume.r#type = Some(VolumeType::from(value.as_ref()));
                }
                _ => {}
            }
        }
        
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"triangle" => {
                            let triangle = Triangle::parse(reader)?;
                            volume.triangles.push(triangle);
                        }
                        b"metadata" => {
                            let metadata = Metadata::parse(reader, e)?;
                            volume.metadatas.push(metadata);
                        }
                        b"color" => {
                            let color = Color::parse(reader)?;
                            volume.colors.push(color);
                        }
                        _ => {},
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"volume" => break,
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(volume)
    }
}

impl Triangle {
    // Parse triangle element
    fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut triangle = Self::default();
        let mut buf = Vec::new();
        let mut current: Option<String> = None;
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"texmap" => {
                            triangle.texmap = Some(TexMap::parse(reader, e)?);
                        }
                        b"color" => {
                            let color = Color::parse(reader)?;
                            triangle.colors.push(color);
                        }
                        _ => {
                            current = Some(String::from_utf8_lossy(e.name().as_ref()).to_string());
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if let Some(ref tag) = current {
                        let content = e.decode()?;
                        let text = unescape(&content)?.to_string();
                        let value: u32 = text.trim().parse().unwrap_or(0);
                        match tag.as_str() {
                            "v1" => triangle.v1 = value,
                            "v2" => triangle.v2 = value,
                            "v3" => triangle.v3 = value,
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if e.name().as_ref() == b"triangle" {
                        break;
                    }
                    current = None;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(triangle)
    }
}

impl TexMap {
    // Parse texmap element
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut texmap = Self::default();
        
        // Parse attributes
        for attr in elem.attributes().flatten() {
            match attr.key.as_ref() {
                b"rtexid" => {
                    let value = attr.unescape_value()?;
                    texmap.rtexid = Some(value.parse().unwrap_or(0));
                }
                b"gtexid" => {
                    let value = attr.unescape_value()?;
                    texmap.gtexid = Some(value.parse().unwrap_or(0));
                }
                b"btexid" => {
                    let value = attr.unescape_value()?;
                    texmap.btexid = Some(value.parse().unwrap_or(0));
                }
                b"atexid" => {
                    let value = attr.unescape_value()?;
                    texmap.atexid = Some(value.parse().unwrap_or(0));
                }
                _ => {}
            }
        }
        
        let mut buf = Vec::new();
        let mut current: Option<String> = None;
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    current = Some(String::from_utf8_lossy(e.name().as_ref()).to_string());
                }
                Ok(Event::Text(ref e)) => {
                    if let Some(ref tag) = current {
                        let content = e.decode()?;
                        let text = unescape(&content)?.to_string();
                        let value: f64 = text.trim().parse().unwrap_or(0.0);
                        match tag.as_str() {
                            "utex1" => texmap.utex1 = value,
                            "utex2" => texmap.utex2 = value,
                            "utex3" => texmap.utex3 = value,
                            "vtex1" => texmap.vtex1 = value,
                            "vtex2" => texmap.vtex2 = value,
                            "vtex3" => texmap.vtex3 = value,
                            "wtex1" => texmap.wtex1 = Some(value),
                            "wtex2" => texmap.wtex2 = Some(value),
                            "wtex3" => texmap.wtex3 = Some(value),
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if e.name().as_ref() == b"texmap" {
                        break;
                    }
                    current = None;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(texmap)
    }
}

impl Color {
    // Parse color element
    fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut color = Self::default();
        let mut buf = Vec::new();
        let mut current: Option<String> = None;
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    current = Some(String::from_utf8_lossy(e.name().as_ref()).to_string());
                }
                Ok(Event::Text(ref e)) => {
                    if let Some(ref tag) = current {
                        let content = e.decode()?;
                        let text = unescape(&content)?.to_string();
                        match tag.as_str() {
                            "r" => color.r = text.trim().to_string(),
                            "g" => color.g = text.trim().to_string(),
                            "b" => color.b = text.trim().to_string(),
                            "a" => color.a = Some(text.trim().to_string()),
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if e.name().as_ref() == b"color" {
                        break;
                    }
                    current = None;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(color)
    }
}

impl Metadata {
    // Parse metadata element
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut metadata = Self::default();
        
        // Parse type attribute
        for attr in elem.attributes().flatten() {
            if attr.key.as_ref() == b"type" {
                metadata.r#type = attr.unescape_value()?.to_string();
            }
        }
        
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Text(ref e)) => {
                    let content = e.decode()?;
                    let text = unescape(&content)?.to_string();
                    metadata.value = text.trim().to_string();
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"metadata" => break,
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(metadata)
    }
}

impl Material {
    // Parse material element
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut material = Self::new(0);
        
        // Parse id attribute
        for attr in elem.attributes().flatten() {
            if attr.key.as_ref() == b"id" {
                let value = attr.unescape_value()?;
                material.id = value.parse().unwrap_or(0);
            }
        }
        
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"metadata" => {
                            let metadata = Metadata::parse(reader, e)?;
                            material.metadatas.push(metadata);
                        }
                        b"composite" => {
                            let composite = Composite::parse(reader, e)?;
                            material.composites.push(composite);
                        }
                        b"color" => {
                            // Materials can have color element, skip for now
                        }
                        _ => {},
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"material" => break,
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(material)
    }
}

impl Composite {
    // Parse composite element
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut composite = Self::default();
        
        // Parse materialid attribute
        for attr in elem.attributes().flatten() {
            if attr.key.as_ref() == b"materialid" {
                let value = attr.unescape_value()?;
                composite.material_id = value.parse().unwrap_or(0);
            }
        }
        
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Text(ref e)) => {
                    let content = e.decode()?;
                    let text = unescape(&content)?.to_string();
                    composite.value = text.trim().to_string();
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"composite" => break,
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(composite)
    }
}

impl Texture {
    // Parse texture element
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut texture = Self::default();
        
        // Parse attributes
        for attr in elem.attributes().flatten() {
            match attr.key.as_ref() {
                b"id" => {
                    let value = attr.unescape_value()?;
                    texture.id = value.parse().unwrap_or(0);
                }
                b"width" => {
                    let value = attr.unescape_value()?;
                    texture.width = value.parse().unwrap_or(0);
                }
                b"height" => {
                    let value = attr.unescape_value()?;
                    texture.height = value.parse().unwrap_or(0);
                }
                b"depth" => {
                    let value = attr.unescape_value()?;
                    texture.depth = value.parse().unwrap_or(1);
                }
                b"tiled" => {
                    let value = attr.unescape_value()?;
                    texture.tiled = value.to_lowercase() == "true" || value == "1";
                }
                b"type" => {
                    let value = attr.unescape_value()?;
                    texture.r#type = TextureType::from(value.as_ref());
                }
                _ => {}
            }
        }
        
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Text(ref e)) => {
                    let content = e.decode()?;
                    let text = unescape(&content)?.to_string();
                    texture.value = text.trim().to_string();
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"texture" => break,
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(texture)
    }
}

impl Constellation {
    // Parse constellation element
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut constellation = Self::default();
        
        // Parse id attribute
        for attr in elem.attributes().flatten() {
            if attr.key.as_ref() == b"id" {
                let value = attr.unescape_value()?;
                constellation.id = value.parse().unwrap_or(0);
            }
        }
        
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    if e.name().as_ref() == b"instance" {
                        let instance = Instance::parse(reader, e)?;
                        constellation.instances.push(instance);
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    if e.name().as_ref() == b"instance" {
                        let instance = Instance::attributes(e)?;
                        constellation.instances.push(instance);
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"constellation" => break,
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(constellation)
    }
}

impl Instance {
    // Parse instance element
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut instance = Self::attributes(elem)?;
        
        let mut buf = Vec::new();
        let mut current: Option<String> = None;
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    current = Some(String::from_utf8_lossy(e.name().as_ref()).to_string());
                }
                Ok(Event::Text(ref e)) => {
                    if let Some(ref tag) = current {
                        let content = e.decode()?;
                        let text = unescape(&content)?.to_string();
                        let value: f64 = text.trim().parse().unwrap_or(0.0);
                        match tag.as_str() {
                            "deltax" => instance.deltax = Some(value),
                            "deltay" => instance.deltay = Some(value),
                            "deltaz" => instance.deltaz = Some(value),
                            "rx" => instance.rx = Some(value),
                            "ry" => instance.ry = Some(value),
                            "rz" => instance.rz = Some(value),
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if e.name().as_ref() == b"instance" {
                        break;
                    }
                    current = None;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(instance)
    }

    // Parse instance from start element (for empty elements)
    fn attributes(elem: &BytesStart) -> Result<Self> {
        let mut instance = Self::default();
        
        for attr in elem.attributes().flatten() {
            match attr.key.as_ref() {
                b"objectid" => {
                    let value = attr.unescape_value()?;
                    instance.object_id = value.parse().unwrap_or(0);
                }
                b"deltax" => {
                    let value = attr.unescape_value()?;
                    instance.deltax = Some(value.parse().unwrap_or(0.0));
                }
                b"deltay" => {
                    let value = attr.unescape_value()?;
                    instance.deltay = Some(value.parse().unwrap_or(0.0));
                }
                b"deltaz" => {
                    let value = attr.unescape_value()?;
                    instance.deltaz = Some(value.parse().unwrap_or(0.0));
                }
                b"rx" => {
                    let value = attr.unescape_value()?;
                    instance.rx = Some(value.parse().unwrap_or(0.0));
                }
                b"ry" => {
                    let value = attr.unescape_value()?;
                    instance.ry = Some(value.parse().unwrap_or(0.0));
                }
                b"rz" => {
                    let value = attr.unescape_value()?;
                    instance.rz = Some(value.parse().unwrap_or(0.0));
                }
                _ => {}
            }
        }
        
        Ok(instance)
    }
}

/// Read AMF document from a file path
/// Automatically detects whether the file is XML or ZIP format
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<Document> {
    let file = File::open(path.as_ref())?;
    let mut reader = BufReader::new(file);
    
    // Read first few bytes to detect format
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    
    // Reopen file to read from beginning
    let file = File::open(path.as_ref())?;
    let reader = BufReader::new(file);
    
    // ZIP files start with PK (0x50, 0x4B)
    if magic[0] == 0x50 && magic[1] == 0x4B {
        read_zip(reader)
    } else {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        read_xml(&content)
    }
}

/// Read AMF document from bytes
/// Supports both XML and ZIP formats
pub fn read_bytes(data: &[u8]) -> Result<Document> {
    // Check if it's a ZIP file
    if data.len() >= 4 && data[0] == 0x50 && data[1] == 0x4B {
        let cursor = Cursor::new(data);
        read_zip(cursor)
    } else {
        let xml = str::from_utf8(data)?;
        read_xml(xml)
    }
}

/// Read AMF document from XML string
pub fn read_xml(xml: &str) -> Result<Document> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    
    let mut document = Document::new();
    let mut buf = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"amf" => {
                document.attributes(e)?;
                document.parse(&mut reader)?;
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Error::Xml(e)),
            _ => {}
        }
        buf.clear();
    }
    
    Ok(document)
}
/// Read AMF document from ZIP archive
pub fn read_zip<R: Read + Seek>(reader: R) -> Result<Document> {
    let mut archive = ZipArchive::new(reader)?;
    
    // Find the AMF file in the archive
    let mut content = String::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_lowercase();
        if name.ends_with(".amf") || name.ends_with(".xml") {
            file.read_to_string(&mut content)?;
            break;
        }
    }
    
    if content.is_empty() {
        // If no .amf file found, try to read the first file
        if !archive.is_empty() {
            let mut file = archive.by_index(0)?;
            file.read_to_string(&mut content)?;
        }
    }
    
    read_xml(&content)
}