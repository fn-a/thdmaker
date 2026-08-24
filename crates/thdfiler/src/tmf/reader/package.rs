use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Seek};
use quick_xml::events::Event;
use quick_xml::Reader;
use zip::read::ZipArchive;
use glob::Pattern;
use crate::util::path::*;
use super::error::{Error, Result};
use super::define::package::*;
use super::define::model::*;
use super::define::securecontent::KeyStore;

impl Package {
    /// Read a 3MF package from a reader.
    pub fn read<R: Read + Seek>(reader: R) -> Result<Self> {
        let mut archive = ZipArchive::new(reader)?;
        let mut package = Self::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.is_dir() {  continue; }

            let name = file.name().to_string();
            let path = absolute(&name, None);

            // Parse content types
            if path == Pathway::CONTENT_TYPES_PATH {
                package.set_content_types(ContentTypes::parse(BufReader::new(file))?);
                continue;
            }
            
            // Parse relationships
            if path == Pathway::RELS_ROOT_PATH {
                package.set_relationships(Relationships::parse(BufReader::new(file))?);
                continue;
            }
            
            // Parse thumbnail
            if path.starts_with(Pathway::THUMBNAIL_PATH) {
                let mut data = Vec::new();
                file.read_to_end(&mut data)?;
                package.add_attachment(name, data);
                continue;
            }

            // Parse model
            if path == Pathway::MODEL_PATH {
                let model = Model::parse(BufReader::new(file))?;
                package.set_model(model);
                continue;
            }

            // Parse other relationships
            if  Pattern::new(Pathway::RELS_GLOB_EXT)?.matches(&path) {
                package.add_relationships(name, Relationships::parse(BufReader::new(file))?);
                continue;
            }

            // Parse other models
            if Pattern::new(Pathway::MODEL_GLOB_EXT)?.matches(&path) {
                package.add_model(name, Model::parse(BufReader::new(file))?);
                continue;
            }

            // Parse key store for secure content extension
            if path == Pathway::KEY_STORE_PATH {
                let keystore = KeyStore::parse(BufReader::new(file))?;
                package.set_key_store(keystore);
                continue;
            }

            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            package.add_attachment(name, data);
        }

        Ok(package)
    }
}

impl Relationships {
    /// Parse relationships XML.
    pub fn parse<R: BufRead>(reader: R) -> Result<Self> {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);
        
        let mut relationships = Vec::new();
        let mut buf = Vec::new();
    
        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) if e.name().as_ref() == b"Relationship" => {
                    let mut id = String::new();
                    let mut rel_type = String::new();
                    let mut target = String::new();
    
                    for attr in e.attributes().flatten() {
                        let value = attr.unescape_value().unwrap_or_default();
                        match attr.key.as_ref() {
                            b"Id" => id = value.to_string(),
                            b"Type" => rel_type = value.to_string(),
                            b"Target" => target = value.to_string(),
                            _ => {}
                        }
                    }
    
                    relationships.push(Relationship { id, r#type: rel_type, target });
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }
    
        Ok(Self { relationships })
    }
}

impl ContentTypes {
    /// Parse content types XML.
    pub fn parse<R: BufRead>(reader: R) -> Result<Self> {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);
        
        let mut defaults = Vec::new();
        let mut overrides = Vec::new();
        let mut unknowns: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();
        let mut buf = Vec::new();
    
        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) => {
                    match e.name().as_ref() {
                        b"Default" => {
                            let mut extension = String::new();
                            let mut content_type = String::new();
                            for attr in e.attributes().flatten() {
                                let value = attr.unescape_value().unwrap_or_default();
                                match attr.key.as_ref() {
                                    b"Extension" => extension = value.to_string(),
                                    b"ContentType" => content_type = value.to_string(),
                                    _ => {}
                                }
                            }
                            if !extension.is_empty() {
                                defaults.push(ContentTypeDefault {
                                    extension,
                                    content_type,
                                });
                            }
                        }
                        b"Override" => {
                            let mut part_name = String::new();
                            let mut content_type = String::new();
                            for attr in e.attributes().flatten() {
                                let value = attr.unescape_value().unwrap_or_default();
                                match attr.key.as_ref() {
                                    b"PartName" => part_name = value.to_string(),
                                    b"ContentType" => content_type = value.to_string(),
                                    _ => {}
                                }
                            }
                            if !part_name.is_empty() {
                                overrides.push(ContentTypeOverride {
                                    part_name: normalize(&part_name, None),
                                    content_type,
                                });
                            }
                        }
                        n => {
                            let mut attrs = HashMap::new();
                            for attr in e.attributes().flatten() {
                                let value = attr.unescape_value().unwrap_or_default();
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                
                                attrs.insert(key, value.to_string());
                            }
                            let name = String::from_utf8_lossy(n).to_string();
                            unknowns.entry(name)
                                .or_default()
                                .push(attrs);
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }
    
        Ok(Self {
            defaults,
            overrides,
            unknowns,
        })
    }
}