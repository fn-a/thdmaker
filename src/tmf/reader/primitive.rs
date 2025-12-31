use std::io::BufRead;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use super::error::{Error, Result};
use super::define::primitive::*;
use super::define::trianglesets::*;
use super::define::beamlattice::*;

impl Mesh {
    pub fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut mesh = Mesh::new();
        for attr in elem.attributes().flatten() {
            if attr.key.as_ref() == b"volumeid" {
                mesh.volume_id = Some(attr.unescape_value()?.parse()?);
            }
        }
        
        let mut buf = Vec::new();
    
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.local_name().as_ref() {
                        b"vertices" => {
                            mesh.vertices = Vertices::parse(reader)?;
                        }
                        b"triangles" => {
                            mesh.triangles = Triangles::parse(reader)?;
                        }
                         b"trianglesets" => {
                            mesh.triangle_sets = TriangleSets::parse(reader)?;
                        }
                        b"beamlattice" => {
                            mesh.beam_lattice = Some(BeamLattice::parse(reader, e)?);
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"mesh" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("mesh".to_string())),
                Ok(_) => {} // Skip unknown elements (like triangle sets from extensions)
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }
    
        Ok(mesh)
    }
}

impl Vertices {
    fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut vertices = Vec::new();
        let mut buf = Vec::new();
    
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) if e.name().as_ref() == b"vertex" => {
                    let vertex = Vertex::parse(e)?;
                    vertices.push(vertex);
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"vertices" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("vertices".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }
    
        Ok(Self { vertices })
    }
}

impl Vertex {
    fn parse(elem: &BytesStart) -> Result<Self> {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;
    
        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;
    
            match key {
                b"x" => x = value.parse()?,
                b"y" => y = value.parse()?,
                b"z" => z = value.parse()?,
                _ => {}
            }
        }
    
        Ok(Self::new(x, y, z))
    }
}

impl Triangles {
    fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut triangles = Vec::new();
        let mut buf = Vec::new();
    
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) if e.name().as_ref() == b"triangle" => {
                    let triangle = Triangle::parse(e)?;
                    triangles.push(triangle);
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"triangles" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("triangles".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }
    
        Ok(Self { triangles })
    }
}

impl Triangle {
    fn parse(elem: &BytesStart) -> Result<Self> {
        let mut triangle = Self::default();
    
        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;
    
            match key {
                b"v1" => triangle.v1 = value.parse()?,
                b"v2" => triangle.v2 = value.parse()?,
                b"v3" => triangle.v3 = value.parse()?,
                b"p1" => triangle.p1 = Some(value.parse()?),
                b"p2" => triangle.p2 = Some(value.parse()?),
                b"p3" => triangle.p3 = Some(value.parse()?),
                b"pid" => triangle.pid = Some(value.parse()?),
                _ => {}
            }
        }
    
        Ok(triangle)
    }
}