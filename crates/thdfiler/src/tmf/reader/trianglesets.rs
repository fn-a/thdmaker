use std::io::BufRead;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use super::error::{Error, Result};
use super::define::trianglesets::*;

impl TriangleSets {
    pub fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut triangle_sets = Vec::new();
        let mut buf = Vec::new();
    
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"triangleset" => {
                    let set = TriangleSet::parse(reader, e)?;
                    triangle_sets.push(set);
                }
                Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"triangleset" => {
                    let set = TriangleSet::attributes(e)?;
                    triangle_sets.push(set);
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"trianglesets" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("trianglesets".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }
    
        Ok(Self { triangle_sets })
    }
}

impl TriangleSet {
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut set = Self::attributes(elem)?;
        let mut buf = Vec::new();
    
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"ref" => {
                    let r = TriangleRef::parse(e)?;
                    set.r#ref.push(r);
                }
                Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"refrange" => {
                    let range = TriangleRefRange::parse(e)?;
                    set.refrange.push(range);
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"triangleset" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("triangleset".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }
    
        Ok(set)
    }

    fn attributes(elem: &BytesStart) -> Result<Self> {
        let mut name = String::new();
        let mut identifier = String::new();
    
        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;
    
            match key {
                b"name" => name = value.to_string(),
                b"identifier" => identifier = value.to_string(),
                _ => {}
            }
        }
    
        Ok(Self {
            name,
            identifier,
            r#ref: Vec::new(),
            refrange: Vec::new(),
        })
    }
}

impl TriangleRef {
    fn parse(elem: &BytesStart) -> Result<Self> {
        let mut index = 0usize;
    
        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;
    
            if key == b"index" {
                index = value.parse()?;
            }
        }
    
        Ok(Self { index })
    }
}

impl TriangleRefRange {
    fn parse(elem: &BytesStart) -> Result<Self> {
        let mut start_index = 0usize;
        let mut end_index = 0usize;
    
        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;
    
            match key {
                b"startindex" => start_index = value.parse()?,
                b"endindex" => end_index = value.parse()?,
                _ => {}
            }
        }
    
        Ok(Self { start_index, end_index })
    }
}