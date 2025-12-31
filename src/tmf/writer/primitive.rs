use std::io::Write;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use super::error::Result;
use super::define::primitive::*;

impl Mesh {
    pub fn write<W: Write>(self: &Self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("mesh");
        if let Some(volume_id) = self.volume_id {
            elem.push_attribute(("volumeid", volume_id.to_string().as_str()));
        }

        writer.write_event(Event::Start(elem))?;
    
        // Vertices
        writer.write_event(Event::Start(BytesStart::new("vertices")))?;
        for vertex in &self.vertices.vertices {
            let mut elem = BytesStart::new("vertex");
            elem.push_attribute(("x", vertex.x.to_string().as_str()));
            elem.push_attribute(("y", vertex.y.to_string().as_str()));
            elem.push_attribute(("z", vertex.z.to_string().as_str()));
            writer.write_event(Event::Empty(elem))?;
        }
        writer.write_event(Event::End(BytesEnd::new("vertices")))?;
    
        // Triangles
        writer.write_event(Event::Start(BytesStart::new("triangles")))?;
        for triangle in &self.triangles.triangles {
            let mut elem = BytesStart::new("triangle");
            elem.push_attribute(("v1", triangle.v1.to_string().as_str()));
            elem.push_attribute(("v2", triangle.v2.to_string().as_str()));
            elem.push_attribute(("v3", triangle.v3.to_string().as_str()));
            
            if let Some(pid) = triangle.pid {
                elem.push_attribute(("pid", pid.to_string().as_str()));
            }
            if let Some(p1) = triangle.p1 {
                elem.push_attribute(("p1", p1.to_string().as_str()));
            }
            if let Some(p2) = triangle.p2 {
                elem.push_attribute(("p2", p2.to_string().as_str()));
            }
            if let Some(p3) = triangle.p3 {
                elem.push_attribute(("p3", p3.to_string().as_str()));
            }
            
            writer.write_event(Event::Empty(elem))?;
        }
        writer.write_event(Event::End(BytesEnd::new("triangles")))?;
    
        // Triangle sets (if any)
        if !self.triangle_sets.triangle_sets.is_empty() {
            self.triangle_sets.write(writer)?;
        }

        // Beam lattice (if any)
        if let Some(ref beam_lattice) = self.beam_lattice {
            beam_lattice.write(writer)?
        }
    
        writer.write_event(Event::End(BytesEnd::new("mesh")))?;

        Ok(())
    }
}