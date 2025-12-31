use std::io::Write;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use super::error::Result;
use super::define::slice::*;

impl SliceStack {
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("s:slicestack");
        elem.push_attribute(("id", self.id.to_string().as_str()));
        
        if let Some(z_bottom) = self.z_bottom {
            elem.push_attribute(("zbottom", z_bottom.to_string().as_str()));
        }

        writer.write_event(Event::Start(elem))?;

        match &self.content {
            SliceContent::Slices(slices) => {
                for slice in slices {
                    slice.write(writer)?;
                }
            }
            SliceContent::SliceRefs(slice_refs) => {
                for slice_ref in slice_refs {
                    slice_ref.write(writer)?;
                }
            }
        }

        writer.write_event(Event::End(BytesEnd::new("s:slicestack")))?;

        Ok(())
    }
}

impl SliceRef {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("s:sliceref");
        elem.push_attribute(("slicestackid", self.slice_stack_id.to_string().as_str()));
        elem.push_attribute(("slicepath", self.slice_path.as_str()));
        writer.write_event(Event::Empty(elem))?;
        Ok(())
    }
}

impl Slice {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("s:slice");
        elem.push_attribute(("ztop", self.z_top.to_string().as_str()));

        if self.vertices.vertices.is_empty() && self.polygons.is_empty() {
            writer.write_event(Event::Empty(elem))?;
        } else {
            writer.write_event(Event::Start(elem))?;

            self.vertices.write(writer)?;

            for polygon in &self.polygons {
                polygon.write(writer)?;
            }

            writer.write_event(Event::End(BytesEnd::new("s:slice")))?;
        }

        Ok(())
    }
}

impl Vertices2D {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        if !self.vertices.is_empty() {
            let elem = BytesStart::new("s:vertices");
            writer.write_event(Event::Start(elem))?;
            for vertex in &self.vertices {
                vertex.write(writer)?;
            }
            writer.write_event(Event::End(BytesEnd::new("s:vertices")))?;
        }
        Ok(())
    }
}

impl Vertex2D {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("s:vertex");
        elem.push_attribute(("x", self.x.to_string().as_str()));
        elem.push_attribute(("y", self.y.to_string().as_str()));
        writer.write_event(Event::Empty(elem))?;
        Ok(())
    }
}

impl Polygon {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("s:polygon");
        elem.push_attribute(("startv", self.start_v.to_string().as_str()));

        if self.segments.is_empty() {
            writer.write_event(Event::Empty(elem))?;
        } else {
            writer.write_event(Event::Start(elem))?;
            for segment in &self.segments {
                segment.write(writer)?;
            }
            writer.write_event(Event::End(BytesEnd::new("s:polygon")))?;
        }

        Ok(())
    }
}

impl Segment {
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("s:segment");
        elem.push_attribute(("v2", self.v2.to_string().as_str()));

        if let Some(p1) = self.p1 {
            elem.push_attribute(("p1", p1.to_string().as_str()));
        }
        if let Some(p2) = self.p2 {
            elem.push_attribute(("p2", p2.to_string().as_str()));
        }
        if let Some(pid) = self.pid {
            elem.push_attribute(("pid", pid.to_string().as_str()));
        }

        writer.write_event(Event::Empty(elem))?;

        Ok(())
    }
}

impl SliceResources {
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        for slice_stack in &self.slice_stacks {
            slice_stack.write(writer)?;
        }
        Ok(())
    }
}

impl SliceObject {
    pub fn write(&self, elem: &mut BytesStart) -> Result<()> {
        if let Some(id) = self.slice_stack_id {
            elem.push_attribute(("s:slicestackid", id.to_string().as_str()));
        }
        if let Some(res) = self.mesh_resolution {
            elem.push_attribute(("s:meshresolution", res.to_string().as_str()));
        }
        Ok(())
    }
}