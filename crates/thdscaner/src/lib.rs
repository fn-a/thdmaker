//! 3D scanning: turning captures of a real object into a 3D digital model.

/// The stages a scan goes through, from captures to a mesh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stage {
    /// Photographs, or a depth capture, of the object from every side.
    Capture,
    /// Camera poses and a sparse point cloud, from structure from motion.
    Align,
    /// A dense reconstruction of the surface, as a point cloud or a splat.
    Dense,
    /// A closed triangle mesh: the digital model itself.
    Surface,
}

impl Stage {
    /// The stage that follows this one, or `None` at the end of the pipeline.
    pub fn next(self) -> Option<Self> {
        match self {
            Self::Capture => Some(Self::Align),
            Self::Align => Some(Self::Dense),
            Self::Dense => Some(Self::Surface),
            Self::Surface => None,
        }
    }

    /// Whether the scan has a surface in hand, and nothing is left to run.
    pub fn finished(self) -> bool {
        self.next().is_none()
    }
}

/// One object being scanned: what it is called, and how far along it is.
#[derive(Debug, Clone)]
pub struct Scaner {
    /// What the object is called, for the log and the file a scan is written to.
    name: String,
    /// How far the scan has come.
    stage: Stage,
}

impl Scaner {
    /// Start a scan of an object, at the first stage.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            stage: Stage::Capture,
        }
    }

    /// What the object is called.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// How far the scan has come.
    pub fn stage(&self) -> Stage {
        self.stage
    }

    /// Move on to the next stage, returning it, or `None` when the scan is finished.
    pub fn advance(&mut self) -> Option<Stage> {
        self.stage = self.stage.next()?;
        Some(self.stage)
    }
}
