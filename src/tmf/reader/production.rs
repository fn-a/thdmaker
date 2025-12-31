use std::io::BufRead;
use quick_xml::events::{BytesStart, attributes::Attribute, Event};
use quick_xml::Reader;
use super::error::{Error, Result};
use super::define::production::*;

impl ProductionItem {
    /// Parse production item attributes.
    pub fn parse(attrs: &Vec<Attribute>) -> Result<Self> {
        let mut uuid = None;
        let mut path = None;

        for attr in attrs {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"p:UUID" => uuid = Some(value.to_string()),
                b"p:path" => path = Some(value.to_string()),
                _ => {}
            }
        }

        if let Some(uuid) = uuid {
            let mut item = Self::new(&uuid)?;
            if let Some(path) = path {
                item.set_path(path);
            }
            Ok(item)
        } else {
            Err(Error::InvalidAttribute {
                name: "UUID".to_string(),
                message: "production item requires UUID attribute".to_string(),
            })
        }
    }
}

impl ProductionComponent {
    /// Parse production component attributes.
    pub fn parse(attrs: &Vec<Attribute>) -> Result<Self> {
        let mut uuid = None;
        let mut path = None;

        for attr in attrs {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"p:UUID" => uuid = Some(value.to_string()),
                b"p:path" => path = Some(value.to_string()),
                _ => {}
            }
        }

        if let Some(uuid) = uuid {
            let mut component = Self::new(&uuid)?;
            if let Some(path) = path {
                component.set_path(path);
            }
            Ok(component)
        } else {
            Err(Error::InvalidAttribute {
                name: "UUID".to_string(),
                message: "production component requires UUID attribute".to_string(),
            })
        }
    }
}

impl ProductionObject {
    /// Parse production object attributes.
    pub fn parse(attrs: &Vec<Attribute>) -> Result<Self> {
        let mut uuid = None;
        let mut resolution = ModelResolution::default();

        for attr in attrs {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"p:UUID" => uuid = Some(value.to_string()),
                b"pa:modelresolution" => {
                    resolution = value.parse()?;
                }
                _ => {}
            }
        }

        if let Some(uuid) = uuid {
            Ok(Self::with_resolution(&uuid, resolution)?)
        } else {
            Err(Error::InvalidAttribute {
                name: "UUID".to_string(),
                message: "production object requires UUID attribute".to_string(),
            })
        }
    }
}


impl ProductionBuild {
    /// Parse production build attributes.
    pub fn parse(attrs: &Vec<Attribute>) -> Result<Self> {
        let mut uuid = None;

        for attr in attrs {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            if  key == b"p:UUID" {
                uuid = Some(value.to_string());
            }
        }

        if let Some(uuid) = uuid {
            Ok(Self::new(&uuid)?)
        } else {
            Err(Error::InvalidAttribute {
                name: "UUID".to_string(),
                message: "production build requires UUID attribute".to_string(),
            })
        }
    }
}

impl Alternatives {
    /// Parse alternatives element.
    pub fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut alternatives = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    if e.local_name().as_ref() == b"alternative" {
                        let alternative = Alternative::parse(e)?;
                        alternatives.push(alternative);
                    }
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"alternatives" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("alternatives".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(Self { alternatives })
    }
}

impl Alternative {
    /// Parse alternative element.
    pub fn parse(elem: &BytesStart) -> Result<Self> {
        let mut object_id = 0u32;
        let mut uuid = None;
        let mut path = None;
        let mut model_resolution = ModelResolution::default();

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"objectid" => object_id = value.parse()?,
                b"UUID" => uuid = Some(value.to_string()),
                b"path" => path = Some(value.to_string()),
                b"modelresolution" => {
                    model_resolution = value.parse()?;
                }
                _ => {}
            }
        }

        if object_id == 0 {
            return Err(Error::InvalidAttribute {
                name: "objectid".to_string(),
                message: "alternative requires objectid attribute".to_string(),
            });
        }

        // According the example UUID maybe empty?!
        if let Some(uuid) = uuid {
            let mut alternative = Self::with_resolution(object_id, &uuid, model_resolution)?;
            if let Some(path) = path {
                alternative.set_path(path);
            }
            Ok(alternative)
        } else {
            Err(Error::InvalidAttribute {
                name: "UUID".to_string(),
                message: "alternative requires UUID attribute".to_string(),
            })
        }
    }
}