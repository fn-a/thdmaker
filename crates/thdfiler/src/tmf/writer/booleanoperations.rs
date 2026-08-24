use std::io::Write;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use super::error::Result;
use super::define::booleanoperations::*;

impl BooleanShape {
    /// Write boolean shape element
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("bo:booleanshape");
        
        // Write required attributes
        elem.push_attribute(("objectid", self.object_id.to_string().as_str()));
        
        // Write optional attributes
        if self.operation != BooleanOperation::Union {
            elem.push_attribute(("operation", self.operation.to_string().as_str()));
        }
        
        if let Some(ref transform) = self.transform {
            elem.push_attribute(("transform", transform.to_string().as_str()));
        }
        
        if let Some(ref path) = self.path {
            elem.push_attribute(("path", path.as_str()));
        }
        
        writer.write_event(Event::Start(elem))?;
        
        // Write boolean operation sequence
        for boolean in &self.booleans {
            boolean.write(writer)?;
        }
        
        writer.write_event(Event::End(BytesEnd::new("bo:booleanshape")))?;
        
        Ok(())
    }
}

impl Boolean {
    /// Write boolean operation element
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("bo:boolean");
        
        // Write required attributes
        elem.push_attribute(("objectid", self.object_id.to_string().as_str()));
        
        // Write optional attributes
        if let Some(ref transform) = self.transform {
            elem.push_attribute(("transform", transform.to_string().as_str()));
        }
        
        if let Some(ref path) = self.path {
            elem.push_attribute(("path", path.as_str()));
        }
        
        writer.write_event(Event::Empty(elem))?;
        
        Ok(())
    }
}