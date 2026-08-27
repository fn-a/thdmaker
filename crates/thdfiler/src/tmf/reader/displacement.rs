use std::io::BufRead;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use quick_xml::name::LocalName;
use super::error::{Error, Result};
use super::define::displacement::*;

/// Displacement 2D resource parsing
impl Displacement2D {
    pub fn parse(elem: &BytesStart) -> Result<Self> {
        let mut id = 0u32;
        let mut path = String::new();
        let mut channel = ChannelName::default();
        let mut tile_style_u = TileStyle::default();
        let mut tile_style_v = TileStyle::default();
        let mut filter = Filter::default();

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"id" => id = value.parse()?,
                b"path" => path = value.to_string(),
                b"channel" => channel = value.parse()?,
                b"tilestyleu" => tile_style_u = value.parse()?,
                b"tilestylev" => tile_style_v = value.parse()?,
                b"filter" => filter = value.parse()?,
                _ => {}
            }
        }

        // Validate required attributes
        if id == 0 {
            return Err(Error::InvalidAttribute {
                name: "id".to_string(),
                message: "displacement2d id is required".to_string(),
            });
        }
        if path.is_empty() {
            return Err(Error::InvalidAttribute {
                name: "path".to_string(),
                message: "displacement2d path is required".to_string(),
            });
        }

        Ok(Self {
            id,
            path,
            channel,
            tile_style_u,
            tile_style_v,
            filter,
        })
    }
}

/// Normalized vector group parsing
impl NormVectorGroup {
    pub fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut id = 0u32;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            if key == b"id" {
                id = value.parse()?;
            }
        }

        if id == 0 {
            return Err(Error::InvalidAttribute {
                name: "id".to_string(),
                message: "normvectorgroup id is required".to_string(),
            });
        }

        let mut group = Self::new(id);
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"normvector" => {
                    let normvector = NormVector::parse(e)?;
                    group.norm_vectors.push(normvector);
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"normvectorgroup" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("normvectorgroup".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        if group.norm_vectors.is_empty() {
            return Err(Error::InvalidStructure(
                "normvectorgroup must contain at least one normvector".to_string()
            ));
        }

        Ok(group)
    }
}

/// Normalized vector parsing
impl NormVector {
    fn parse(elem: &BytesStart) -> Result<Self> {
        let mut x = 0.0f64;
        let mut y = 0.0f64;
        let mut z = 0.0f64;

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

/// 2D displacement group parsing
impl Disp2DGroup {
    pub fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut id = 0u32;
        let mut disp_id = 0u32;
        let mut n_id = 0u32;
        let mut height = 0.0f64;
        let mut offset = 0.0f64;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"id" => id = value.parse()?,
                b"dispid" => disp_id = value.parse()?,
                b"nid" => n_id = value.parse()?,
                b"height" => height = value.parse()?,
                b"offset" => offset = value.parse()?,
                _ => {}
            }
        }

        // Validate required attributes
        if id == 0 {
            return Err(Error::InvalidAttribute {
                name: "id".to_string(),
                message: "disp2dgroup id is required".to_string(),
            });
        }
        if disp_id == 0 {
            return Err(Error::InvalidAttribute {
                name: "dispid".to_string(),
                message: "disp2dgroup dispid is required".to_string(),
            });
        }
        if n_id == 0 {
            return Err(Error::InvalidAttribute {
                name: "nid".to_string(),
                message: "disp2dgroup nid is required".to_string(),
            });
        }
        if height == 0.0 {
            return Err(Error::InvalidAttribute {
                name: "height".to_string(),
                message: "disp2dgroup height is required".to_string(),
            });
        }

        let mut group = Self {
            id,
            disp_id,
            n_id,
            height,
            offset,
            disp_2d_coords: Vec::new(),
        };

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"disp2dcoord" => {
                    let coord = Disp2DCoord::parse(e)?;
                    group.disp_2d_coords.push(coord);
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"disp2dgroup" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("disp2dgroup".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        if group.disp_2d_coords.is_empty() {
            return Err(Error::InvalidStructure(
                "disp2dgroup must contain at least one disp2dcoord".to_string()
            ));
        }

        Ok(group)
    }
}

/// 2D displacement coordinate parsing
impl Disp2DCoord {
    fn parse(elem: &BytesStart) -> Result<Self> {
        let mut u = 0.0f64;
        let mut v = 0.0f64;
        let mut n = 0u32;
        let mut f = 1.0f64;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"u" => u = value.parse()?,
                b"v" => v = value.parse()?,
                b"n" => n = value.parse()?,
                b"f" => f = value.parse()?,
                _ => {}
            }
        }

        // Validate required attributes
        if n == 0 {
            return Err(Error::InvalidAttribute {
                name: "n".to_string(),
                message: "disp2dcoord n is required".to_string(),
            });
        }

        Ok(Self::with_factor(u, v, n, f))
    }
}

/// Displacement mesh parsing
impl DisplacementMesh {
    pub fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut displacement_mesh = Self::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"vertices" => {
                    displacement_mesh.vertices = DispVertices::parse(reader)?;
                }
                Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"triangles" => {
                    displacement_mesh.triangles = DispTriangles::parse(reader, e)?;
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"displacementmesh" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("displacementmesh".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        // Validate displacement mesh
        if displacement_mesh.vertex_count() < 3 {
            return Err(Error::InvalidMesh(
                "displacementmesh must have at least 3 vertices".to_string()
            ));
        }
        if displacement_mesh.triangle_count() < 1 {
            return Err(Error::InvalidMesh(
                "displacementmesh must have at least 1 triangle".to_string()
            ));
        }

        Ok(displacement_mesh)
    }
}

/// Displacement vertex collection parsing
impl DispVertices {
    fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut vertices = Self::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"vertex" => {
                    let vertex = DispVertex::parse(e)?;
                    vertices.vertices.push(vertex);
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"vertices" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("vertices".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        if vertices.vertices.len() < 3 {
            return Err(Error::InvalidMesh(
                "vertices must contain at least 3 vertices".to_string()
            ));
        }

        Ok(vertices)
    }
}

/// Displacement vertex parsing
impl DispVertex {
    fn parse(elem: &BytesStart) -> Result<Self> {
        let mut x = 0.0f64;
        let mut y = 0.0f64;
        let mut z = 0.0f64;

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

/// Displacement triangle collection parsing
impl DispTriangles {
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut triangles = Self::new();
        let mut buf = Vec::new();

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            if key == b"did" {
                triangles.did = Some(value.parse()?);
            }
        }

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"triangle" => {
                    let triangle = DispTriangle::parse(e)?;
                    triangles.triangles.push(triangle);
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"triangles" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("triangles".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        if triangles.triangles.is_empty() {
            return Err(Error::InvalidMesh(
                "triangles must contain at least 1 triangle".to_string()
            ));
        }

        Ok(triangles)
    }
}

/// Displacement triangle parsing
impl DispTriangle {
    fn parse(elem: &BytesStart) -> Result<Self> {
        let mut v1 = 0u32;
        let mut v2 = 0u32;
        let mut v3 = 0u32;
        let mut d1 = None;
        let mut d2 = None;
        let mut d3 = None;
        let mut did = None;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"v1" => v1 = value.parse()?,
                b"v2" => v2 = value.parse()?,
                b"v3" => v3 = value.parse()?,
                b"d1" => d1 = Some(value.parse()?),
                b"d2" => d2 = Some(value.parse()?),
                b"d3" => d3 = Some(value.parse()?),
                b"did" => did = Some(value.parse()?),
                _ => {}
            }
        }

        // Validate vertex indices
        if v1 == 0 || v2 == 0 || v3 == 0 {
            return Err(Error::InvalidAttribute {
                name: "v1/v2/v3".to_string(),
                message: "triangle vertex indices are required".to_string(),
            });
        }

        Ok(Self {
            v1: v1 - 1, // 3MF uses 1-based indexing, convert to 0-based
            v2: v2 - 1,
            v3: v3 - 1,
            d1,
            d2,
            d3,
            did,
        })
    }
}

impl DisplacementResources {
    pub fn parse<R: BufRead>(&mut self, name: &LocalName, reader: &mut Reader<R>, elem: &BytesStart) -> Result<Option<u32>> {
        let mut next_id = None;
        match name.as_ref() {
            b"displacement2d" => {
                let displacement2d = Displacement2D::parse(elem)?;
                next_id = Some(displacement2d.id);
                self.add_displacement_2d(displacement2d);
            }
            b"normvectorgroup" => {
                let normvectorgroup = NormVectorGroup::parse(reader, elem)?;
                next_id = Some(normvectorgroup.id);
                self.add_norm_vector_group(normvectorgroup);
            }
            b"disp2dgroup" => {
                let disp2dgroup = Disp2DGroup::parse(reader, elem)?;
                next_id = Some(disp2dgroup.id);
                self.add_disp_2d_group(disp2dgroup);
            }
            _ => {}
        }
        Ok(next_id)
    }
}