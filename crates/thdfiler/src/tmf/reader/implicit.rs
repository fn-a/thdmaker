use std::sync::Arc;
use std::io::BufRead;
use std::str::FromStr;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use quick_xml::name::LocalName;
use super::error::{Error, Result};
use super::define::implicit::*;

impl ImplicitFunction {
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut id = 0u32;
        let mut identifier = String::new();
        let mut display_name = None;
        let mut input = Input::default();
        let mut output = Output::default();
        let mut nodes = Vec::new();

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"id" => id = value.parse()?,
                b"identifier" => identifier =  value.to_string(),
                b"displayname" => display_name = Some(value.to_string()),
                _ => {}
            }
        }

        let mut buf = Vec::new();
        // Read child elements
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"in" => input = Input::parse(reader)?,
                        b"out" => output = Output::parse(reader)?,
                        _ => {
                            // Parse node
                            let node = Node::parse(reader, e)?;
                            nodes.push(node);
                        }
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let node = Node::attributes(e)?;
                    nodes.push(node);
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"implicitfunction" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("implicitfunction".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(Self {
            id,
            identifier,
            display_name,
            r#in: input,
            out: output,
            nodes,
        })
    }
}

impl Input {
    fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut values = Vec::new();
        let mut buf = Vec::new();
    
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    values.push(Value::parse(reader, e)?);
                }
                Ok(Event::Empty(ref e)) => {
                    values.push(Value::attributes(e)?);
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"in" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("in".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }
    
        Ok(Self { values })
    }
}

impl Output {
    fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut values = Vec::new();
        let mut buf = Vec::new();
    
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    values.push(Value::parse(reader, e)?);
                }
                Ok(Event::Empty(ref e)) => {
                    values.push(Value::attributes(e)?);
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"out" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("out".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }
    
        Ok(Self { values })
    }
}

impl Value {
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let value = Self::attributes(elem)?;

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::End(_)) => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("functioninput".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(value)
    }

    fn attributes(elem: &BytesStart) -> Result<Self> {
        let mut identifier = String::new();
        let mut display_name = None;
        let mut value_ref = None;

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"identifier" => identifier = value.to_string(),
                b"displayname" => display_name = Some(value.to_string()),
                b"ref" => value_ref = Some(value.to_string()),
                _ => {}
            }
        }

        let name = String::from_utf8_lossy(elem.name().as_ref()).to_string();

        let r#type = if let Some(value_ref) = value_ref {
            ValueType::Ref(ValueRef::from_str(&format!("{name}={value_ref}"))?)
        } else {
            ValueType::Data(ValueData::from_str(&name)?)
        };

        Ok(Self { identifier, display_name, r#type })
    }
}

impl Node {
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut node = Self::attributes(elem)?;

        let mut buf = Vec::new();

        // Read child elements
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"in" => node.r#in = Input::parse(reader)?,
                        b"out" => node.out = Output::parse(reader)?,
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == node.r#type.name().as_bytes() => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn(node.r#type.name())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(node)
    }

    fn attributes(elem: &BytesStart) -> Result<Self> {
        let mut identifier = String::new();
        let mut display_name = None;
        let mut tag = None;
        let input = Input::default();
        let output = Output::default();
        let mut name = String::from_utf8_lossy(elem.name().as_ref()).to_string();

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"identifier" => identifier = value.to_string(),
                b"displayname" => display_name = Some(value.to_string()),
                b"tag" => tag = Some(value.to_string()),
                _ => {
                    let key = String::from_utf8_lossy(key);
                    name += &format!("|{key}={value}");
                }
            }
        }

        let ntype: Box<dyn NodeType> = name.parse()?;

        Ok(Self { identifier, display_name, tag, r#type: Arc::new(ntype), r#in: input, out: output })
    }
}

impl ImplicitResources {
    /// Parse implicit resources from XML.
    pub fn parse<R: BufRead>(&mut self, name: &LocalName, reader: &mut Reader<R>, elem: &BytesStart) -> Result<Option<u32>> {
        let mut next_id = None;

        match name.as_ref() {
            b"implicitfunction" => {
                let function = ImplicitFunction::parse(reader, elem)?;
                next_id = Some(function.id);
                self.functions.push(function);
            }
            _ => {} // Skip unknown elements
        }

        Ok(next_id)
    }
}