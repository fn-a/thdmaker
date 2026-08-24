use std::io::BufRead;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use super::error::{Error, Result};
use super::define::booleanoperations::*;

impl BooleanShape {
    /// Parse boolean shape element
    pub fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut boolean_shape = Self::attributes(elem)?;

        // 解析布尔形状的子元素（boolean元素）
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"boolean" => {
                    let boolean = Boolean::parse( e)?;
                    boolean_shape.booleans.push(boolean);
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"booleanshape" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("booleanshape".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(boolean_shape)
    }

    fn attributes(elem: &BytesStart) -> Result<Self> {
        let mut object_id = 0u32;
        let mut operation = BooleanOperation::Union;
        let mut transform = None;
        let mut path = None;

        // Parse boolean shape attributes
        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"objectid" => object_id = value.parse()?,
                b"operation" => operation = value.parse()?,
                b"transform" => transform = Some(value.parse()?),
                b"path" => path = Some(value.to_string()),
                _ => {} // Ignore unknown attributes
            }
        }

        // Note: Child element parsing needs to be handled by the upper caller
        Ok(BooleanShape {
            object_id,
            operation,
            transform,
            path,
            booleans: Vec::new(),
        })
    }
}

impl Boolean {
    /// Parse boolean operation element
    pub fn parse(elem: &BytesStart) -> Result<Self> {
        let mut object_id = 0u32;
        let mut transform = None;
        let mut path = None;

        // Parse boolean operation attributes
        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();    
            let value = attr.unescape_value()?;

            match key {
                b"objectid" => object_id = value.parse()?,
                b"transform" => transform = Some(value.parse()?),
                b"path" => path = Some(value.to_string()),
                _ => {} // Ignore unknown attributes
            }
        }

        // Boolean operation element is empty, no child elements
        if object_id == 0 {
            return Err(Error::InvalidAttribute {
                name: "objectid".to_string(),
                message: "boolean operation requires objectid attribute".to_string(),
            });
        }

        Ok(Boolean {
            object_id,
            transform,
            path,
        })
    }
}