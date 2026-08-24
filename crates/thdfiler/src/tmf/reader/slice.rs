use std::io::BufRead;
use quick_xml::events::{BytesStart, attributes::Attribute, Event};
use quick_xml::Reader;
use quick_xml::name::LocalName;
use super::error::{Error, Result};
use super::define::slice::*;

impl SliceStack {
    pub fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut id = 0;
        let mut z_bottom = None;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"id" => id = value.parse()?,
                b"zbottom" => z_bottom = Some(value.parse()?),
                _ => {}
            }
        }

        let mut stack = Self::new(id);
        stack.z_bottom = z_bottom;
        let mut slices = Vec::new();
        let mut refs = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.local_name().as_ref() {
                        b"slice" => {
                            let slice = Slice::parse(reader, e)?;
                            slices.push(slice);
                        }
                        _ => {}
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    match e.local_name().as_ref() {
                        b"sliceref" => {
                            let slice_ref = SliceRef::parse(e)?;
                            refs.push(slice_ref);
                        }
                        b"slice" => {
                            // Empty slice (only ztop)
                            let slice = Slice::attributes(e)?;
                            slices.push(slice);
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"slicestack" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("slicestack".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        if !slices.is_empty() {
            stack.content = SliceContent::Slices(slices);
        } else if !refs.is_empty() {
            stack.content = SliceContent::SliceRefs(refs);
        }

        Ok(stack)
    }
}

impl SliceRef {
    fn parse(elem: &BytesStart) -> Result<Self> {
        let mut slice_stack_id = 0;
        let mut slice_path = String::new();

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"slicestackid" => slice_stack_id = value.parse()?,
                b"slicepath" => slice_path = value.to_string(),
                _ => {}
            }
        }

        Ok(Self {
            slice_stack_id,
            slice_path,
        })
    }
}

impl Slice {
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut slice = Self::attributes(elem)?;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.local_name().as_ref() {
                        b"vertices" => {
                            slice.vertices = Vertices2D::parse(reader)?;
                        }
                        b"polygon" => {
                            let polygon = Polygon::parse(reader, e)?;
                            slice.polygons.push(polygon);
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"slice" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("slice".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(slice)
    }

    fn attributes(elem: &BytesStart) -> Result<Self> {
        let mut z_top = 0.0;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            if key == b"ztop" {
                z_top = value.parse()?;
            }
        }

        Ok(Self::new(z_top))
    }

    
}

impl Vertices2D {
    fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut vertices = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"vertex" => {
                    let vertex = Vertex2D::parse(e)?;
                    vertices.push(vertex);
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"vertices" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("vertices".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(Self { vertices })
    }
}

impl Vertex2D {
    fn parse(elem: &BytesStart) -> Result<Self> {
        let mut x = 0.0;
        let mut y = 0.0;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"x" => x = value.parse()?,
                b"y" => y = value.parse()?,
                _ => {}
            }
        }

        Ok(Self { x, y })
    }
}

impl Polygon {
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut start_v = 0;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            if key == b"startv" {
                start_v = value.parse()?;
            }
        }

        let mut polygon = Self::new(start_v);
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"segment" => {
                    let segment = Segment::parse(e)?;
                    polygon.segments.push(segment);
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"polygon" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("polygon".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(polygon)
    }
}

impl Segment {
    fn parse(elem: &BytesStart) -> Result<Self> {
        let mut v2 = 0;
        let mut p1 = None;
        let mut p2 = None;
        let mut pid = None;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"v2" => v2 = value.parse()?,
                b"p1" => p1 = Some(value.parse()?),
                b"p2" => p2 = Some(value.parse()?),
                b"pid" => pid = Some(value.parse()?),
                _ => {}
            }
        }

        Ok(Self { v2, p1, p2, pid })
    }
}

impl SliceResources {
    pub fn parse<R: BufRead>(&mut self, name: &LocalName, reader: &mut Reader<R>, elem: &BytesStart) -> Result<Option<u32>> {
        let mut next_id = None;

        match name.as_ref() {
            b"slicestack" => {
                let stack = SliceStack::parse(reader, elem)?;
                next_id = Some(stack.id);
                self.slice_stacks.push(stack);
            }
            _ => {}
        }

        Ok(next_id)
    }
}

impl SliceObject {
    pub fn parse(attrs: &Vec<Attribute>) -> Result<Self> {
        let mut data = Self::default();

        for attr in attrs {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"slicestackid" => data.slice_stack_id = Some(value.parse()?),
                b"meshresolution" => data.mesh_resolution = Some(value.parse()?),
                _ => {}
            }
        }

        Ok(data)
    }
}