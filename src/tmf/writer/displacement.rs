use std::io::Write;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use super::error::Result;
use super::define::displacement::*;

/// Displacement 2D resource writing
impl Displacement2D {
    pub fn write<W: Write>(self: &Self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("d:displacement2d");
        elem.push_attribute(("id", self.id.to_string().as_str()));
        elem.push_attribute(("path", self.path.as_str()));
        
        if self.channel != ChannelName::default() {
            elem.push_attribute(("channel", self.channel.to_string().as_str()));
        }
        
        if self.tile_style_u != TileStyle::default() {
            elem.push_attribute(("tilestyleu", self.tile_style_u.to_string().as_str()));
        }
        
        if self.tile_style_v != TileStyle::default() {
            elem.push_attribute(("tilestylev", self.tile_style_v.to_string().as_str()));
        }
        
        if self.filter != Filter::default() {
            elem.push_attribute(("filter", self.filter.to_string().as_str()));
        }

        writer.write_event(Event::Empty(elem))?;

        Ok(())
    }
}

/// Normalized vector group writing
impl NormVectorGroup {
    pub fn write<W: Write>(self: &Self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("d:normvectorgroup");
        elem.push_attribute(("id", self.id.to_string().as_str()));

        writer.write_event(Event::Start(elem))?;

        for vector in &self.norm_vectors {
            let mut vector_elem = BytesStart::new("d:normvector");
            vector_elem.push_attribute(("x", vector.x.to_string().as_str()));
            vector_elem.push_attribute(("y", vector.y.to_string().as_str()));
            vector_elem.push_attribute(("z", vector.z.to_string().as_str()));
            writer.write_event(Event::Empty(vector_elem))?;
        }

        writer.write_event(Event::End(BytesEnd::new("d:normvectorgroup")))?;

        Ok(())
    }
}

/// 2D displacement group writing
impl Disp2DGroup {
    pub fn write<W: Write>(self: &Self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("d:disp2dgroup");
        elem.push_attribute(("id", self.id.to_string().as_str()));
        elem.push_attribute(("dispid", self.disp_id.to_string().as_str()));
        elem.push_attribute(("nid", self.n_id.to_string().as_str()));
        elem.push_attribute(("height", self.height.to_string().as_str()));
        
        if self.offset != 0.0 {
            elem.push_attribute(("offset", self.offset.to_string().as_str()));
        }

        writer.write_event(Event::Start(elem))?;

        for coord in &self.disp_2d_coords {
            let mut coord_elem = BytesStart::new("d:disp2dcoord");
            coord_elem.push_attribute(("u", coord.u.to_string().as_str()));
            coord_elem.push_attribute(("v", coord.v.to_string().as_str()));
            coord_elem.push_attribute(("n", coord.n.to_string().as_str()));
            
            if coord.f != 1.0 {
                coord_elem.push_attribute(("f", coord.f.to_string().as_str()));
            }
            
            writer.write_event(Event::Empty(coord_elem))?;
        }

        writer.write_event(Event::End(BytesEnd::new("d:disp2dgroup")))?;

        Ok(())
    }
}

/// Displacement mesh writing
impl DisplacementMesh {
    pub fn write<W: Write>(self: &Self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("d:displacementmesh")))?;

        // Write vertices
        self.vertices.write(writer)?;

        // Write triangles
        self.triangles.write(writer)?;

        writer.write_event(Event::End(BytesEnd::new("d:displacementmesh")))?;

        Ok(())
    }
}

/// Displacement vertex collection writing
impl DispVertices {
    fn write<W: Write>(self: &Self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("d:vertices")))?;

        for vertex in &self.vertices {
            let mut vertex_elem = BytesStart::new("d:vertex");
            vertex_elem.push_attribute(("x", vertex.x.to_string().as_str()));
            vertex_elem.push_attribute(("y", vertex.y.to_string().as_str()));
            vertex_elem.push_attribute(("z", vertex.z.to_string().as_str()));
            writer.write_event(Event::Empty(vertex_elem))?;
        }

        writer.write_event(Event::End(BytesEnd::new("d:vertices")))?;

        Ok(())
    }
}

/// Displacement triangle collection writing
impl DispTriangles {
    fn write<W: Write>(self: &Self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("d:triangles");
        
        if let Some(did) = self.did {
            elem.push_attribute(("did", did.to_string().as_str()));
        }

        writer.write_event(Event::Start(elem))?;

        for triangle in &self.triangles {
            triangle.write(writer)?;
        }

        writer.write_event(Event::End(BytesEnd::new("d:triangles")))?;

        Ok(())
    }
}

/// Displacement triangle writing
impl DispTriangle {
    fn write<W: Write>(self: &Self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("d:triangle");
        
        // 3MF uses 1-based indexing
        elem.push_attribute(("v1", (self.v1 + 1).to_string().as_str()));
        elem.push_attribute(("v2", (self.v2 + 1).to_string().as_str()));
        elem.push_attribute(("v3", (self.v3 + 1).to_string().as_str()));
        
        if let Some(did) = self.did {
            elem.push_attribute(("did", did.to_string().as_str()));
        }
        
        if let Some(d1) = self.d1 {
            elem.push_attribute(("d1", d1.to_string().as_str()));
        }
        
        if let Some(d2) = self.d2 {
            elem.push_attribute(("d2", d2.to_string().as_str()));
        }
        
        if let Some(d3) = self.d3 {
            elem.push_attribute(("d3", d3.to_string().as_str()));
        }

        writer.write_event(Event::Empty(elem))?;

        Ok(())
    }
}

impl DisplacementResources {
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        for disp2dgroup in &self.disp_2d_groups {
            disp2dgroup.write(writer)?;
        }

        for normvectorgroup in &self.norm_vector_groups {
            normvectorgroup.write(writer)?;
        }

        for disp2d in &self.displacement_2ds {
            disp2d.write(writer)?;
        }

        Ok(())
    }
}