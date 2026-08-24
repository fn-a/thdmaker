//! Beam Lattice extension.
//!
//! This module implements the 3MF Beam Lattice Extension specification which adds
//! support for lattice structures and truss-based geometries in 3D manufacturing.

use std::fmt;
use std::str::FromStr;
use super::error::{Error, Result};

/// Beam lattice structure for representing lattice geometries.
#[derive(Debug, Clone, Default)]
pub struct BeamLattice {
    /// Minimum length of all beams in the lattice.
    pub min_length: f64,
    /// Default uniform radius value for the beams.
    pub radius: f64,
    /// Ball mode specifying whether balls are created at beam vertices.
    pub ball_mode: BallMode,
    /// Default uniform radius value for the balls.
    pub ball_radius: Option<f64>,
    /// Clipping mode of the beam lattice.
    pub clipping_mode: ClippingMode,
    /// Resource ID of the clipping mesh.
    pub clipping_mesh: Option<u32>,
    /// Resource ID of the representation mesh.
    pub representation_mesh: Option<u32>,
    /// Default property group ID for all beams.
    pub pid: Option<u32>,
    /// Default property index for all beams.
    pub pindex: Option<u32>,
    /// Default capping mode for beam ends.
    pub cap: CapMode,
    /// Collection of beams.
    pub beams: Beams,
    /// Collection of balls.
    pub balls: Balls,
    /// Collection of beam sets.
    pub beam_sets: BeamSets,
}

impl BeamLattice {
    /// Create a new beam lattice with minimum length and radius.
    pub fn new(minlength: f64, radius: f64) -> Self {
        Self {
            min_length: minlength,
            radius,
            ball_mode: BallMode::None,
            ball_radius: None,
            clipping_mode: ClippingMode::None,
            clipping_mesh: None,
            representation_mesh: None,
            pid: None,
            pindex: None,
            cap: CapMode::Sphere,
            beams: Beams::default(),
            balls: Balls::default(),
            beam_sets: BeamSets::default(),
        }
    }

    /// Add a beam to the lattice.
    pub fn add_beam(&mut self, beam: Beam) -> u32 {
        let index = self.beams.beams.len() as u32;
        self.beams.beams.push(beam);
        index
    }

    /// Add a ball to the lattice.
    pub fn add_ball(&mut self, ball: Ball) -> u32 {
        let index = self.balls.balls.len() as u32;
        self.balls.balls.push(ball);
        index
    }

    /// Add a beam set to the lattice.
    pub fn add_beamset(&mut self, beamset: BeamSet) -> u32 {
        let index = self.beam_sets.beam_sets.len() as u32;
        self.beam_sets.beam_sets.push(beamset);
        index
    }

    /// Set the ball mode.
    pub fn set_ball_mode(&mut self, ballmode: BallMode) -> &mut Self {
        self.ball_mode = ballmode;
        self
    }

    /// Set the ball radius.
    pub fn set_ball_radius(&mut self, ballradius: f64) -> &mut Self {
        self.ball_radius = Some(ballradius);
        self
    }

    /// Set the clipping mode.
    pub fn set_clipping_mode(&mut self, clippingmode: ClippingMode) -> &mut Self {
        self.clipping_mode = clippingmode;
        self
    }

    /// Set the clipping mesh resource ID.
    pub fn set_clipping_mesh(&mut self, clippingmesh: u32) -> &mut Self {
        self.clipping_mesh = Some(clippingmesh);
        self
    }

    /// Set the representation mesh resource ID.
    pub fn set_representation_mesh(&mut self, representationmesh: u32) -> &mut Self {
        self.representation_mesh = Some(representationmesh);
        self
    }

    /// Set the default material.
    pub fn set_material(&mut self, pid: u32, pindex: u32) -> &mut Self {
        self.pid = Some(pid);
        self.pindex = Some(pindex);
        self
    }

    /// Set the default cap mode.
    pub fn set_cap_mode(&mut self, cap: CapMode) -> &mut Self {
        self.cap = cap;
        self
    }

    /// Get the number of beams.
    pub fn beam_count(&self) -> usize {
        self.beams.beams.len()
    }

    /// Get the number of balls.
    pub fn ball_count(&self) -> usize {
        self.balls.balls.len()
    }

    /// Get the number of beam sets.
    pub fn beam_set_count(&self) -> usize {
        self.beam_sets.beam_sets.len()
    }
}

/// Clipping mode for beam lattice geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClippingMode {
    /// No clipping applied.
    #[default]
    None,
    /// Keep geometry inside the clipping mesh volume.
    Inside,
    /// Keep geometry outside the clipping mesh volume.
    Outside,
}

impl fmt::Display for ClippingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClippingMode::None => write!(f, "none"),
            ClippingMode::Inside => write!(f, "inside"),
            ClippingMode::Outside => write!(f, "outside"),
        }
    }
}

impl FromStr for ClippingMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "none" => Ok(ClippingMode::None),
            "inside" => Ok(ClippingMode::Inside),
            "outside" => Ok(ClippingMode::Outside),
            _ => Err(Error::InvalidAttribute {
                name: "clippingmode".to_string(),
                message: format!("unknown clipping mode: {}", s),
            }),
        }
    }
}

/// Cap mode for beam ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CapMode {
    /// Half-sphere cap.
    Hemisphere,
    /// Full sphere cap.
    #[default]
    Sphere,
    /// Flat end cap.
    Butt,
}

impl fmt::Display for CapMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CapMode::Hemisphere => write!(f, "hemisphere"),
            CapMode::Sphere => write!(f, "sphere"),
            CapMode::Butt => write!(f, "butt"),
        }
    }
}

impl FromStr for CapMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "hemisphere" => Ok(CapMode::Hemisphere),
            "sphere" => Ok(CapMode::Sphere),
            "butt" => Ok(CapMode::Butt),
            _ => Err(Error::InvalidAttribute {
                name: "capmode".to_string(),
                message: format!("unknown cap mode: {}", s),
            }),
        }
    }
}

/// Ball mode for beam lattice vertices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BallMode {
    /// No balls created at vertices.
    #[default]
    None,
    /// Balls created for vertices with explicit ball elements.
    Mixed,
    /// Balls created at all beam end vertices.
    All,
}

impl fmt::Display for BallMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BallMode::None => write!(f, "none"),
            BallMode::Mixed => write!(f, "mixed"),
            BallMode::All => write!(f, "all"),
        }
    }
}

impl FromStr for BallMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "none" => Ok(BallMode::None),
            "mixed" => Ok(BallMode::Mixed),
            "all" => Ok(BallMode::All),
            _ => Err(Error::InvalidAttribute {
                name: "ballmode".to_string(),
                message: format!("unknown ball mode: {}", s),
            }),
        }
    }
}

/// A collection of beams in a beam lattice.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Beams {
    /// List of beams.
    pub beams: Vec<Beam>,
}

/// A single beam in a beam lattice.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Beam {
    /// Index of first vertex.
    pub v1: u32,
    /// Index of second vertex.
    pub v2: u32,
    /// Radius at first vertex.
    pub r1: Option<f64>,
    /// Radius at second vertex.
    pub r2: Option<f64>,
    /// Property index for first vertex.
    pub p1: Option<u32>,
    /// Property index for second vertex.
    pub p2: Option<u32>,
    /// Property group ID.
    pub pid: Option<u32>,
    /// Cap mode for first vertex.
    pub cap1: Option<CapMode>,
    /// Cap mode for second vertex.
    pub cap2: Option<CapMode>,
}

impl Beam {
    /// Create a new beam between two vertices.
    pub fn new(v1: u32, v2: u32) -> Self {
        Self {
            v1,
            v2,
            r1: None,
            r2: None,
            p1: None,
            p2: None,
            pid: None,
            cap1: None,
            cap2: None,
        }
    }

    /// Create a beam with radius at both ends.
    pub fn with_radius(v1: u32, v2: u32, r1: f64, r2: f64) -> Self {
        Self {
            v1,
            v2,
            r1: Some(r1),
            r2: Some(r2),
            p1: None,
            p2: None,
            pid: None,
            cap1: None,
            cap2: None,
        }
    }

    /// Set the property for both vertices.
    pub fn with_property(mut self, pid: u32, pindex: u32) -> Self {
        self.pid = Some(pid);
        self.p1 = Some(pindex);
        self.p2 = Some(pindex);
        self
    }
}

/// A collection of balls in a beam lattice.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Balls {
    /// List of balls.
    pub balls: Vec<Ball>,
}

/// A ball element in a beam lattice.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ball {
    /// Index of vertex that serves as center.
    pub vindex: u32,
    /// Radius of the ball.
    pub r: Option<f64>,
    /// Property index.
    pub p: Option<u32>,
    /// Property group ID.
    pub pid: Option<u32>,
}

impl Ball {
    /// Create a new ball at a vertex.
    pub fn new(vindex: u32) -> Self {
        Self {
            vindex,
            r: None,
            p: None,
            pid: None,
        }
    }

    /// Create a ball with radius.
    pub fn with_radius(vindex: u32, r: f64) -> Self {
        Self {
            vindex,
            r: Some(r),
            p: None,
            pid: None,
        }
    }

    /// Set the property.
    pub fn with_property(mut self, pid: u32, pindex: u32) -> Self {
        self.pid = Some(pid);
        self.p = Some(pindex);
        self
    }
}

/// A reference to a beam in a beam set.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BeamRef {
    /// Index of the beam.
    pub index: u32,
}

impl BeamRef {
    /// Create a new beam reference.
    pub fn new(index: u32) -> Self {
        Self { index }
    }
}

/// A reference to a ball in a beam set.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BallRef {
    /// Index of the ball.
    pub index: u32,
}

impl BallRef {
    /// Create a new ball reference.
    pub fn new(index: u32) -> Self {
        Self { index }
    }
}

/// A collection of beam sets in a beam lattice.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BeamSets {
    /// List of beam sets.
    pub beam_sets: Vec<BeamSet>,
}

/// A beam set for grouping beams and balls.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BeamSet {
    /// Human-readable name.
    pub name: Option<String>,
    /// Unique identifier.
    pub identifier: Option<String>,
    /// Beam references.
    pub refs: Vec<BeamRef>,
    /// Ball references.
    pub ball_refs: Vec<BallRef>,
}

impl BeamSet {
    /// Create a new empty beam set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a named beam set.
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            identifier: None,
            refs: Vec::new(),
            ball_refs: Vec::new(),
        }
    }

    /// Add a beam reference.
    pub fn add_beam_ref(&mut self, index: u32) {
        self.refs.push(BeamRef::new(index));
    }

    /// Add a ball reference.
    pub fn add_ball_ref(&mut self, index: u32) {
        self.ball_refs.push(BallRef::new(index));
    }
}