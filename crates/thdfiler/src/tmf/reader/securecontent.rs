use std::str::FromStr;
use std::io::BufRead;
use quick_xml::events::{BytesStart, Event};
use quick_xml::escape::unescape;
use quick_xml::Reader;
use super::error::{Error, Result};
use super::define::securecontent::*;

impl KeyStore {
    pub fn parse<R: BufRead>(reader: R) -> Result<Self> {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);

        let mut keystore = Self::default();
        let mut buf = Vec::new();

        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"keystore" => {
                    keystore = Self::attributes(e)?;

                    let mut buf = Vec::new();
                    loop {
                        match xml_reader.read_event_into(&mut buf) {
                            Ok(Event::Start(ref e)) => {
                                match e.name().as_ref() {
                                    b"consumer" => {
                                        let consumer = Consumer::parse(&mut xml_reader, e)?;
                                        keystore.consumers.push(consumer);
                                    }
                                    b"resourcedatagroup" => {
                                        let group = ResourceDataGroup::parse(&mut xml_reader, e)?;
                                        keystore.resource_data_groups.push(group);
                                    }
                                    _ => {}
                                }
                            }
                            Ok(Event::Empty(ref e)) => {
                                match e.name().as_ref() {
                                    b"consumer" => {
                                        let consumer = Consumer::attributes(e)?;
                                        keystore.consumers.push(consumer);
                                    }
                                    _ => {}
                                }
                            }
                            Ok(_) => {}
                            Err(e) => return Err(Error::Xml(e)),
                        }
                        buf.clear();
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"keystore" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("keystore".to_string())),
                Ok(_) => {} // Skip other events
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(keystore)
    }

    fn attributes(elem: &BytesStart) -> Result<Self> {
        let mut uuid = None;
        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;
            
            if key == b"UUID" {
                uuid = Some(value.to_string());
            }
        }

        if let Some(uuid) = uuid {
            Ok(Self::new(&uuid)?)
        } else {
            Err(Error::InvalidAttribute {
                name: "UUID".to_string(),
                message: "keystore requires UUID attribute".to_string(),
            })
        }
    }
}

impl Consumer {
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut consumer = Self::attributes(elem)?;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    if e.name().as_ref() == b"keyvalue" {
                        consumer.key_value = KeyValue::parse(reader)?;
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"consumer" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("consumer".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(consumer)
    }

    fn attributes(elem: &BytesStart) -> Result<Self> {
        let mut consumer = Self::default();

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"consumerid" => consumer.consumer_id = value.to_string(),
                b"keyid" => consumer.key_id = Some(value.to_string()),
                _ => {}
            }
        }

        Ok(consumer)
    }
}

impl KeyValue {
    fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Option<Self>> {
        let mut buf = Vec::new();
        let mut value = None;
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Text(e)) => {
                    let content = e.decode()?;
                    value = Some(Self(unescape(&content)?.to_string()));
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"keyvalue" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("keyvalue".to_string())),
                _ => return Err(Error::InvalidStructure("keyvalue requires text content".to_string())),
            }
            buf.clear();
        }

        Ok(value)
    }
}

impl ResourceDataGroup {
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut resource = Self::attributes(elem)?;

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"accessright" => {
                            let access_right = AccessRight::parse(reader, e)?;
                            resource.access_rights.push(access_right);
                        }
                        b"resourcedata" => {
                            let resource_data = ResourceData::parse(reader, e)?;
                            resource.resource_datas.push(resource_data);
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"resourcedatagroup" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("resourcedatagroup".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(resource)
    }

    fn attributes(elem: &BytesStart) -> Result<Self> {
        let mut uuid = None;
        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;
            
            if key == b"UUID" {
                uuid = Some(value.to_string());
            }
        }

        if let Some(uuid) = uuid {
            Ok(Self::new(&uuid)?)
        } else {
            Err(Error::InvalidAttribute {
                name: "UUID".to_string(),
                message: "resourcedatagroup requires UUID attribute".to_string(),
            })
        }
    }
}

impl ResourceData {
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut resource_data = Self::default();

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            if key == b"path" {
                resource_data.path = value.to_string();
            }
        }

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    if e.name().as_ref() == b"cekparams" {
                        resource_data.cek_params = CEKParams::parse(reader, e)?;
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"resourcedata" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("resourcedata".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(resource_data)
    }
}

impl AccessRight {
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut access_right = Self::default();

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            if key == b"consumerindex" {
                access_right.consumer_index = value.parse()?;
            }
        }

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"cipherdata" => {
                            access_right.cipher_data = CipherData::parse(reader)?;
                        }
                        _ => {}
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    if e.name().as_ref() == b"kekparams" {
                        access_right.kek_params = KEKParams::attributes(e)?;
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"accessright" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("accessright".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(access_right)
    }
}

impl CEKParams {
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut params = Self::default();

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"encryptionalgorithm" => params.encryption_algorithm = value.to_string(),
                b"compression" => params.compression = Compression::from_str(&value)?,
                _ => {}
            }
        }

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"iv" => params.iv = Some(Base64Value::parse(reader, b"iv")?),
                        b"tag" => params.tag = Some(Base64Value::parse(reader, b"tag")?),
                        b"aad" => params.aad = Some(Base64Value::parse(reader, b"aad")?),
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"cekparams" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("cekparams".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(params)
    }
}

impl KEKParams {
    fn attributes(elem: &BytesStart) -> Result<Self> {
        let mut params = Self::default();

        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;

            match key {
                b"wrappingalgorithm" => params.wrapping_algorithm = value.to_string(),
                b"mgfalgorithm" => params.mgf_algorithm = Some(value.to_string()),
                b"digestmethod" => params.digest_method = Some(value.to_string()),
                _ => {}
            }
        }

        Ok(params)
    }
}

impl CipherData {
    fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut cipher_data = Self::default();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    if e.name().as_ref() == b"xenc:CipherValue" {
                        cipher_data.cipher_value = CipherValue::parse(reader)?;
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"cipherdata" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("cipherdata".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }

        Ok(cipher_data)
    }
}

impl CipherValue {
    fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        Ok(Self(Base64Value::parse(reader, b"xenc:CipherValue")?))
    }
}

impl Base64Value {
    fn parse<R: BufRead>(reader: &mut Reader<R>, name: &[u8]) -> Result<Self> {
        let mut buf = Vec::new();
        let mut content = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Text(e)) => {
                    content = unescape(&e.decode()?)?.to_string();
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == name => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn(String::from_utf8_lossy(name).to_string())),
                _ => {}
            }
            buf.clear();
        }

        content.as_str().parse()
    }
}