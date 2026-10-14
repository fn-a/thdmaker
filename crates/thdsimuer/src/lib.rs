//! 3D simulation: verifying a 3D model by simulating it in its intended environment.

/// The stages a simulation goes through, from a model to a verified verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stage {
    /// The model and the scenario it must survive: loads, drops, heat, and tolerances.
    Import,
    /// The checks that run before any physics: watertight, manifold, within printable bounds.
    Validate,
    /// The scenario stepped over time: how the model deforms, moves, and holds up.
    Simulate,
    /// The measurements compared against the tolerances, and the verdict drawn.
    Verify,
}

impl Stage {
    /// The stage that follows this one, or `None` at the end of the pipeline.
    pub fn next(self) -> Option<Self> {
        match self {
            Self::Import => Some(Self::Validate),
            Self::Validate => Some(Self::Simulate),
            Self::Simulate => Some(Self::Verify),
            Self::Verify => None,
        }
    }

    /// Whether the simulation has a verdict in hand, and nothing is left to run.
    pub fn finished(self) -> bool {
        self.next().is_none()
    }
}

/// The outcome of a verified simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Verdict {
    /// Every tolerance held, and the model is fit for its purpose.
    Pass,
    /// A tolerance was broken, and the model must go back to the drawing board.
    Fail,
}

/// One model being simulated: what it is called, how far along it is, and how it held up.
#[derive(Debug, Clone)]
pub struct Simuer {
    /// What the model is called, for the log and the report a simulation is written to.
    name: String,
    /// How far the simulation has come.
    stage: Stage,
    /// The verdict, once the model has been verified.
    verdict: Option<Verdict>,
}

impl Simuer {
    /// Start a simulation of a model, at the first stage.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            stage: Stage::Import,
            verdict: None,
        }
    }

    /// What the model is called.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// How far the simulation has come.
    pub fn stage(&self) -> Stage {
        self.stage
    }

    /// The verdict, once the model has been verified.
    pub fn verdict(&self) -> Option<Verdict> {
        self.verdict
    }

    /// Move on to the next stage, returning it, or `None` when the simulation is finished.
    pub fn advance(&mut self) -> Option<Stage> {
        self.stage = self.stage.next()?;
        Some(self.stage)
    }

    /// Record the verdict, once the simulation has been verified.
    pub fn conclude(&mut self, verdict: Verdict) -> Verdict {
        self.verdict = Some(verdict);
        verdict
    }
}
