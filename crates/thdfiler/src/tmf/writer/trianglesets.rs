use std::io::Write;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use super::error::Result;
use super::define::package::*;
use super::define::trianglesets::*;

impl TriangleSets {
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut ts_elem = BytesStart::new("t:trianglesets");
        ts_elem.push_attribute(("xmlns:t", Namespace::TRIANGLESETS_NS));
        writer.write_event(Event::Start(ts_elem))?;
    
        for trgl_set in &self.triangle_sets {
            let mut set_elem = BytesStart::new("t:triangleset");
            set_elem.push_attribute(("name", trgl_set.name.as_str()));
            set_elem.push_attribute(("identifier", trgl_set.identifier.as_str()));
            writer.write_event(Event::Start(set_elem))?;
    
            // Write refs (could optimize with refrange later)
            for &trgl_ref in &trgl_set.r#ref {
                let mut ref_elem = BytesStart::new("t:ref");
                ref_elem.push_attribute(("index", trgl_ref.index.to_string().as_str()));
                writer.write_event(Event::Empty(ref_elem))?;
            }

            for &trgl_range in &trgl_set.refrange {
                let mut ref_range_elem = BytesStart::new("t:refrange");
                ref_range_elem.push_attribute(("startindex", trgl_range.start_index.to_string().as_str()));
                ref_range_elem.push_attribute(("endindex", trgl_range.end_index.to_string().as_str()));
                writer.write_event(Event::Empty(ref_range_elem))?;
            }
    
            writer.write_event(Event::End(BytesEnd::new("t:triangleset")))?;
        }
    
        writer.write_event(Event::End(BytesEnd::new("t:trianglesets")))?;

        Ok(())
    }
}