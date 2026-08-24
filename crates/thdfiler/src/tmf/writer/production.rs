use std::io::Write;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use super::error::Result;
use super::define::package::Namespace;
use super::define::production::*;

impl ProductionItem {
    /// Write production item attributes.
    pub fn write(&self, elem: &mut BytesStart) -> Result<()> {
        elem.push_attribute(("p:UUID", self.uuid.to_string().as_str()));
        if let Some(ref path) = self.path {
            elem.push_attribute(("p:path", path.as_str()));
        }
        Ok(())
    }
}

impl ProductionBuild {
    /// Write production build attributes.
    pub fn write(&self, elem: &mut BytesStart) -> Result<()> {
        elem.push_attribute(("p:UUID", self.uuid.to_string().as_str()));
        Ok(())
    }
}

impl ProductionComponent {
    /// Write production component attributes.
    pub fn write(&self, elem: &mut BytesStart) -> Result<()> {
        elem.push_attribute(("p:UUID", self.uuid.to_string().as_str()));
        if let Some(ref path) = self.path {
            elem.push_attribute(("p:path", path.as_str()));
        }
        Ok(())
    }
}

impl ProductionObject {
    /// Write production object attributes.
    pub fn write(&self, elem: &mut BytesStart) -> Result<()> {
        elem.push_attribute(("p:UUID", self.uuid.to_string().as_str()));
        if self.model_resolution != ModelResolution::default() {
            elem.push_attribute(("pa:modelresolution", self.model_resolution.to_string().as_str()));
        }
        Ok(())
    }
}

impl Alternatives {
    /// Write alternatives element.
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        if !self.is_empty() {
            let mut elem = BytesStart::new("pa:alternatives");
            elem.push_attribute(("xmlns:pa", Namespace::ALTERNATIVES_NS));
            writer.write_event(Event::Start(elem))?;
    
            for alternative in &self.alternatives {
                alternative.write(writer)?;
            }
    
            writer.write_event(Event::End(BytesEnd::new("pa:alternatives")))?;
        }

        Ok(())
    }
}

impl Alternative {
    /// Write alternative element.
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("pa:alternative");
        elem.push_attribute(("objectid", self.object_id.to_string().as_str()));
        elem.push_attribute(("UUID", self.uuid.to_string().as_str()));

        if let Some(ref path) = self.path {
            elem.push_attribute(("path", path.as_str()));
        }

        if self.model_resolution != ModelResolution::default() {
            elem.push_attribute(("modelresolution", self.model_resolution.to_string().as_str()));
        }

        writer.write_event(Event::Empty(elem))?;
        Ok(())
    }
}