use std::io::Write;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use super::error::Result;
use super::define::package::Namespace;
use super::define::beamlattice::*;

impl BeamLattice {
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut bl_elem = BytesStart::new("b:beamlattice");
        bl_elem.push_attribute(("xmlns:b", Namespace::BEAMLATTICE_NS));
        bl_elem.push_attribute(("minlength", self.min_length.to_string().as_str()));
        bl_elem.push_attribute(("radius", self.radius.to_string().as_str()));
    
        if self.ball_mode != BallMode::None {
            bl_elem.push_attribute(("xmlns:b2", Namespace::BALLS_NS));
            bl_elem.push_attribute(("b2:ballmode", self.ball_mode.to_string().as_str()));
            if let Some(ballradius) = self.ball_radius {
                bl_elem.push_attribute(("b2:ballradius", ballradius.to_string().as_str()));
            }
        }
    
        if self.clipping_mode != ClippingMode::None {
            bl_elem.push_attribute(("clippingmode", self.clipping_mode.to_string().as_str()));
            if let Some(clipping_mesh) = self.clipping_mesh {
                bl_elem.push_attribute(("clippingmesh", clipping_mesh.to_string().as_str()));
            }
        }
    
        if let Some(representation_mesh) = self.representation_mesh {
            bl_elem.push_attribute(("representationmesh", representation_mesh.to_string().as_str()));
        }
    
        if let Some(pid) = self.pid {
            bl_elem.push_attribute(("pid", pid.to_string().as_str()));
        }
    
        if let Some(pindex) = self.pindex {
            bl_elem.push_attribute(("pindex", pindex.to_string().as_str()));
        }
    
        if self.cap != CapMode::Sphere {
            bl_elem.push_attribute(("cap", self.cap.to_string().as_str()));
        }
    
        writer.write_event(Event::Start(bl_elem))?;
    
        // Write beams
        self.beams.write(writer)?;
    
        // Write balls if any
        if !self.balls.balls.is_empty() {
            self.balls.write(writer)?;
        }
        
        // Write beam sets if any
        if !self.beam_sets.beam_sets.is_empty() {
            self.beam_sets.write(writer)?;
        }
    
        writer.write_event(Event::End(BytesEnd::new("b:beamlattice")))?;
        Ok(())
    }
}

impl Beams {
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("b:beams")))?;
    
        for beam in &self.beams {
            let mut beam_elem = BytesStart::new("b:beam");
            beam_elem.push_attribute(("v1", beam.v1.to_string().as_str()));
            beam_elem.push_attribute(("v2", beam.v2.to_string().as_str()));
    
            if let Some(r1) = beam.r1 {
                beam_elem.push_attribute(("r1", r1.to_string().as_str()));
            }
    
            if let Some(r2) = beam.r2 {
                beam_elem.push_attribute(("r2", r2.to_string().as_str()));
            }
    
            if let Some(p1) = beam.p1 {
                beam_elem.push_attribute(("p1", p1.to_string().as_str()));
            }
    
            if let Some(p2) = beam.p2 {
                beam_elem.push_attribute(("p2", p2.to_string().as_str()));
            }
    
            if let Some(pid) = beam.pid {
                beam_elem.push_attribute(("pid", pid.to_string().as_str()));
            }
    
            if let Some(cap1) = beam.cap1 {
                beam_elem.push_attribute(("cap1", cap1.to_string().as_str()));
            }
    
            if let Some(cap2) = beam.cap2 {
                beam_elem.push_attribute(("cap2", cap2.to_string().as_str()));
            }
    
            writer.write_event(Event::Empty(beam_elem))?;
        }
    
        writer.write_event(Event::End(BytesEnd::new("b:beams")))?;
        Ok(())
    }
}

impl Balls {
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("b2:balls")))?;

        for ball in &self.balls {
            let mut ball_elem = BytesStart::new("b2:ball");
            ball_elem.push_attribute(("vindex", ball.vindex.to_string().as_str()));

            if let Some(r) = ball.r {
                ball_elem.push_attribute(("r", r.to_string().as_str()));
            }

            if let Some(p) = ball.p {
                ball_elem.push_attribute(("p", p.to_string().as_str()));
            }

            if let Some(pid) = ball.pid {
                ball_elem.push_attribute(("pid", pid.to_string().as_str()));
            }

            writer.write_event(Event::Empty(ball_elem))?;
        }

        writer.write_event(Event::End(BytesEnd::new("b2:balls")))?;
        Ok(())
    }
}

impl BeamSets {
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("b:beamsets")))?;

        for beamset in &self.beam_sets {
            let mut bs_elem = BytesStart::new("b:beamset");

            if let Some(ref name) = beamset.name {
                bs_elem.push_attribute(("name", name.as_str()));
            }

            if let Some(ref identifier) = beamset.identifier {
                bs_elem.push_attribute(("identifier", identifier.as_str()));
            }

            writer.write_event(Event::Start(bs_elem))?;

            // Write beam references
            for beam_ref in &beamset.refs {
                let mut ref_elem = BytesStart::new("b:ref");
                ref_elem.push_attribute(("index", beam_ref.index.to_string().as_str()));
                writer.write_event(Event::Empty(ref_elem))?;
            }

            // Write ball references
            for ball_ref in &beamset.ball_refs {
                let mut ballref_elem = BytesStart::new("b2:ballref");
                ballref_elem.push_attribute(("index", ball_ref.index.to_string().as_str()));
                writer.write_event(Event::Empty(ballref_elem))?;
            }

            writer.write_event(Event::End(BytesEnd::new("b:beamset")))?;
        }

        writer.write_event(Event::End(BytesEnd::new("b:beamsets")))?;
        Ok(())
    }
}