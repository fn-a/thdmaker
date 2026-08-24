use std::io::BufRead;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use super::error::{Error, Result};
use super::define::beamlattice::*;

impl BeamLattice {
    pub fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut minlength = 0.0;
        let mut radius = 0.0;
        let mut ballmode = BallMode::None;
        let mut ballradius = None;
        let mut clippingmode = ClippingMode::None;
        let mut clippingmesh = None;
        let mut representationmesh = None;
        let mut pid = None;
        let mut pindex = None;
        let mut cap = CapMode::Sphere;
    
        // Parse attributes
        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;
    
            match key {
                b"minlength" => minlength = value.parse()?,
                b"radius" => radius = value.parse()?,
                b"b2:ballmode" => ballmode = value.parse()?,
                b"b2:ballradius" => ballradius = Some(value.parse()?),
                b"clippingmode" => clippingmode = value.parse()?,
                b"clippingmesh" => clippingmesh = Some(value.parse()?),
                b"representationmesh" => representationmesh = Some(value.parse()?),
                b"pid" => pid = Some(value.parse()?),
                b"pindex" => pindex = Some(value.parse()?),
                b"cap" => cap = value.parse()?,
                _ => {} // Ignore unknown attributes
            }
        }
    
        let mut beam_lattice = BeamLattice {
            min_length: minlength,
            radius,
            ball_mode: ballmode,
            ball_radius: ballradius,
            clipping_mode: clippingmode,
            clipping_mesh: clippingmesh,
            representation_mesh: representationmesh,
            pid,
            pindex,
            cap,
            beams: Beams::default(),
            balls: Balls::default(),
            beam_sets: BeamSets::default(),
        };
    
        let mut buf = Vec::new();
    
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"beams" => {
                    beam_lattice.beams = Beams::parse(reader)?;
                }
                Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"balls" => {
                    beam_lattice.balls = Balls::parse(reader)?;
                }
                Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"beamsets" => {
                    beam_lattice.beam_sets = BeamSets::parse(reader)?;
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"beamlattice" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("beamlattice".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }
    
        Ok(beam_lattice)
    }
}

impl Beams {
    fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut beams = Vec::new();
        let mut buf = Vec::new();
    
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"beam" => {
                    let beam = Beam::parse(e)?;
                    beams.push(beam);
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"beams" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("beams".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }
    
        Ok(Self{ beams })
    }
}

impl Beam {
    fn parse(elem: &BytesStart) -> Result<Self> {
        let mut beam = Self::default();
    
        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;
    
            match key {
                b"v1" => beam.v1 = value.parse()?,
                b"v2" => beam.v2 = value.parse()?,
                b"r1" => beam.r1 = Some(value.parse()?),
                b"r2" => beam.r2 = Some(value.parse()?),
                b"p1" => beam.p1 = Some(value.parse()?),
                b"p2" => beam.p2 = Some(value.parse()?),
                b"pid" => beam.pid = Some(value.parse()?),
                b"cap1" => beam.cap1 = Some(value.parse()?),
                b"cap2" => beam.cap2 = Some(value.parse()?),
                _ => {}
            }
        }
    
        Ok(beam)
    }
}

impl Balls {
    fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut balls = Vec::new();
        let mut buf = Vec::new();
    
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"ball" => {
                    let ball = Ball::parse(e)?;
                    balls.push(ball);
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"balls" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("balls".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }
    
        Ok(Self{ balls })
    }
}

impl Ball {
    fn parse(elem: &BytesStart) -> Result<Self> {
        let mut ball = Self::default();
    
        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;
    
            match key {
                b"vindex" => ball.vindex = value.parse()?,
                b"r" => ball.r = Some(value.parse()?),
                b"p" => ball.p = Some(value.parse()?),
                b"pid" => ball.pid = Some(value.parse()?),
                _ => {}
            }
        }
    
        Ok(ball)
    }
}

impl BeamSets {
    fn parse<R: BufRead>(reader: &mut Reader<R>) -> Result<Self> {
        let mut beam_sets = Vec::new();
        let mut buf = Vec::new();
    
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"beamset" => {
                    let beamset = BeamSet::parse(reader, e)?;
                    beam_sets.push(beamset);
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"beamsets" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("beamsets".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }
    
        Ok(Self{ beam_sets })
    }
}

impl BeamSet {
    fn parse<R: BufRead>(reader: &mut Reader<R>, elem: &BytesStart) -> Result<Self> {
        let mut name = None;
        let mut identifier = None;
    
        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;
    
            match key {
                b"name" => name = Some(value.to_string()),
                b"identifier" => identifier = Some(value.to_string()),
                _ => {}
            }
        }
    
        let mut beam_set = BeamSet {
            name,
            identifier,
            refs: Vec::new(),
            ball_refs: Vec::new(),
        };
    
        let mut buf = Vec::new();
    
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"ref" => {
                    let beam_ref = BeamRef::parse(e)?;
                    beam_set.refs.push(beam_ref);
                }
                Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"ballref" => {
                    let ball_ref = BallRef::parse(e)?;
                    beam_set.ball_refs.push(ball_ref);
                }
                Ok(Event::End(ref e)) if e.local_name().as_ref() == b"beamset" => break,
                Ok(Event::Eof) => return Err(Error::UnexpectedEofIn("beamset".to_string())),
                Ok(_) => {}
                Err(e) => return Err(Error::Xml(e)),
            }
            buf.clear();
        }
    
        Ok(beam_set)
    }
}

impl BeamRef {
    /// Parse a beam reference from XML.
    fn parse(elem: &BytesStart) -> Result<Self> {
        let mut index = 0u32;
    
        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;
    
            if key == b"index" {
                index = value.parse()?;
            }
        }
    
        Ok(BeamRef { index })
    }
}

impl BallRef {
    /// Parse a ball reference from XML.
    fn parse(elem: &BytesStart) -> Result<Self> {
        let mut index = 0u32;
    
        for attr in elem.attributes().flatten() {
            let key = attr.key.as_ref();
            let value = attr.unescape_value()?;
    
            if key == b"index" {
                index = value.parse()?;
            }
        }
    
        Ok(BallRef { index })
    }
}