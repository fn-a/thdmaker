use std::io::Write;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use super::define::package::Namespace;
use super::error::Result;
use super::define::securecontent::*;

impl KeyStore {
    pub fn write<W: Write>(&self, writer: W) -> Result<()> {
        let mut xml_writer = Writer::new_with_indent(writer, b' ', 2);

        // XML declaration
        xml_writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

        let mut elem = BytesStart::new("keystore");
        elem.push_attribute(("xmlns", Namespace::SECURECONTENT_NS));
        elem.push_attribute(("xmlns:xenc", "http://www.w3.org/2001/04/xmlenc#"));
        elem.push_attribute(("UUID", self.uuid.to_string().as_str()));
        
        xml_writer.write_event(Event::Start(elem))?;

        for consumer in &self.consumers {
            consumer.write(&mut xml_writer)?;
        }

        for group in &self.resource_data_groups {
            group.write(&mut xml_writer)?;
        }

        xml_writer.write_event(Event::End(BytesEnd::new("keystore")))?;
        Ok(())
    }
}

impl Consumer {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("consumer");
        elem.push_attribute(("consumerid", self.consumer_id.as_str()));
        if let Some(key_id) = &self.key_id {
            elem.push_attribute(("keyid", key_id.as_str()));
        }

        if let Some(key_value) = &self.key_value {
            writer.write_event(Event::Start(elem))?;
            key_value.write(writer)?;
            writer.write_event(Event::End(BytesEnd::new("consumer")))?;
        } else {
            writer.write_event(Event::Empty(elem))?;
        }

        Ok(())
    }
}

impl KeyValue {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let key_elem = BytesStart::new("keyvalue");
        writer.write_event(Event::Start(key_elem))?;
        writer.write_event(Event::Text(BytesText::new(&self.0.as_str())))?;
        writer.write_event(Event::End(BytesEnd::new("keyvalue")))?;

        Ok(())
    }
}

impl ResourceDataGroup {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("resourcedatagroup");
        elem.push_attribute(("keyuuid", self.key_uuid.to_string().as_str()));
        
        writer.write_event(Event::Start(elem))?;

        for access_right in &self.access_rights {
            access_right.write(writer)?;
        }

        for resource_data in &self.resource_datas {
            resource_data.write(writer)?;
        }

        writer.write_event(Event::End(BytesEnd::new("resourcedatagroup")))?;
        Ok(())
    }
}

impl ResourceData {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("resourcedata");
        elem.push_attribute(("path", self.path.as_str()));
        
        writer.write_event(Event::Start(elem))?;

        self.cek_params.write(writer)?;

        writer.write_event(Event::End(BytesEnd::new("resourcedata")))?;
        Ok(())
    }
}

impl AccessRight {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("accessright");
        elem.push_attribute(("consumerindex", self.consumer_index.to_string().as_str()));
        
        writer.write_event(Event::Start(elem))?;

        self.kek_params.write(writer)?;
        self.cipher_data.write(writer)?;

        writer.write_event(Event::End(BytesEnd::new("accessright")))?;
        Ok(())
    }
}

impl CEKParams {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("cekparams");
        elem.push_attribute(("encryptionalgorithm", self.encryption_algorithm.as_str()));
        if self.compression != Compression::None {
            elem.push_attribute(("compression", self.compression.to_string().as_str()));
        }
        
        writer.write_event(Event::Start(elem))?;

        if let Some(iv) = &self.iv {
            iv.write(writer, "iv")?;
        }
        if let Some(tag) = &self.tag {
            tag.write(writer, "tag")?;
        }
        if let Some(aad) = &self.aad {
            aad.write(writer, "aad")?;
        }

        writer.write_event(Event::End(BytesEnd::new("cekparams")))?;
        Ok(())
    }
}

impl KEKParams {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("kekparams");
        elem.push_attribute(("wrappingalgorithm", self.wrapping_algorithm.as_str()));
        if let Some(mgf) = &self.mgf_algorithm {
            elem.push_attribute(("mgfalgorithm", mgf.as_str()));
        }
        if let Some(digest) = &self.digest_method {
            elem.push_attribute(("digestmethod", digest.as_str()));
        }
        
        writer.write_event(Event::Empty(elem))?;
        Ok(())
    }
}

impl CipherData {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let elem = BytesStart::new("cipherdata");
        writer.write_event(Event::Start(elem))?;

        self.cipher_value.write(writer)?;

        writer.write_event(Event::End(BytesEnd::new("cipherdata")))?;
        Ok(())
    }
}

impl CipherValue {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        self.0.write(writer, "xenc:CipherValue")
    }
}

impl Base64Value {
    fn write<W: Write>(&self, writer: &mut Writer<W>, name: &str) -> Result<()> {
        let elem = BytesStart::new(name);
        writer.write_event(Event::Start(elem))?;
        writer.write_event(Event::Text(BytesText::new(&self.to_string())))?;
        writer.write_event(Event::End(BytesEnd::new(name)))?;
        Ok(())
    }
}

