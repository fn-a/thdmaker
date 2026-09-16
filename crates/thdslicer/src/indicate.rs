use std::f32::consts::PI;
use crate::gcode::define::*;
use super::Config;
use super::geometry::*;

pub use crate::gcode::define::GCode;

/// The state of the head: where it stands in the layer plane, where its extruder
/// axis stands, and how much of the filament is pulled back right now.
///
/// The driving behaviours live on [`Indicate`] and and work through [`Indicate::coord`],
/// so an implementation keeps the head state without having
/// it handed around, and only renders the moves it is told about. Every drive
/// moves the head the same way, which is what keeps the G-code and the preview
/// describing one toolpath.
#[derive(Debug, Clone, Copy, Default)]
pub struct Coord {
    pub x: f32,
    pub z: f32,
    /// Position of the extruder axis, in millimetres of filament. It is the value
    /// the G-code writes as its `E` word, and it runs backwards while retracting.
    pub extruder: f32,
    /// Filament pulled back, `0` while the filament is not pulled back. The print
    /// after a ride pushes exactly this much back.
    pub retracted: f32,
}

impl Coord {
    /// Whether the head already stands at `point`.
    pub fn at(&self, point: Point2D) -> bool {
        Cov(self.x) == Cov(point.x) && Cov(self.z) == Cov(point.z)
    }
}

/// The drive of the head.
///
/// The methods that end in `-ing` are the driving behaviours: `drawing` prints a
/// segment, `idleing` rides to the start of one, `extruing` and `retracting` move
/// the filament. They work through coord, so an implementation
/// keeps the head state and only renders the moves it is told about, which is
/// what keeps the G-code and the preview on one toolpath.
pub trait Indicate {
    /// Generator start
    fn initiation(&mut self);
    /// Layer begin
    fn laybegin(&mut self, index: usize, len: usize);
    /// Draw line carry: print `segment`.
    ///
    /// A print move starts where the head stands, so a head that is elsewhere
    /// first rides to the start of the segment. That ride is part of printing the
    /// segment, so both the G-code and the preview see it.
    fn drawing(&mut self, segment: &Segment2D) {
        let length = segment.length();
        // A zero length segment has no path to print and no filament to push.
        if length > Cov::<N6>::epow() {
            if !self.coord().at(segment.start) {
                self.idleing(segment);
            }

            self.extruing();
    
            let extrusion = {
                let config = self.config();
                let layheight = if self.index() == 0 {
                    config.first_height
                } else {
                    config.layer_height
                };
                // 1.75mm filament
                let filament = PI * config.filament_width * config.filament_width / 4.0;
                let path_area = config.line_width * layheight;
    
                length * path_area / filament * config.extrusion_multiplier
            };

            {
                let coord = self.coord();
                coord.extruder += extrusion;
                coord.x = segment.end.x;
                coord.z = segment.end.z;
            }
    
            self.drawline(segment);
        }
    }
    /// Draw line implement
    fn drawline(&mut self, segment: &Segment2D);
    /// Idle ride carry: ride to the start of `segment`, without printing.
    fn idleing(&mut self, segment: &Segment2D) {
        self.retracting();

        {
            let coord = self.coord();
            coord.x = segment.start.x;
            coord.z = segment.start.z;
        }

        self.idleride(segment);
    }
    /// Idle ride implement
    fn idleride(&mut self, segment: &Segment2D);
    /// Extrusion carry: push the filament forward again, when printing resumes
    /// after a ride, so the pull back of the ride is undone exactly once.
    fn extruing(&mut self) {
        let enabled = self.config().enable_retraction;
        // The filament goes forward by exactly what the ride pulled back.
        let pulled = self.coord().retracted;
        if enabled && pulled > 0.0 {
            let coord = self.coord();
            coord.extruder += pulled;
            coord.retracted = 0.0;
            self.extrusion();
        }
    }
    /// Extrusion implement
    fn extrusion(&mut self);
    /// Retraction carry: pull the filament back, before the head rides without
    /// printing, and only once for that ride.
    fn retracting(&mut self) {
        let (enabled, distance) = {
            let config = self.config();
            (config.enable_retraction, config.retraction_distance)
        };
        if enabled && self.coord().retracted <= 0.0 {
            let coord = self.coord();
            coord.extruder -= distance;
            coord.retracted = distance;
            self.retraction();
        }
    }
    /// Retraction implement
    fn retraction(&mut self);
    /// Layer finish
    fn layfinish(&mut self);
    /// Completion
    fn completion(&mut self);
    /// Config
    fn config(&self) -> &Config;
    /// Current position
    fn coord(&mut self) -> &mut Coord;
    /// Current layer index
    fn index(&self) -> usize;
}

/// GCode generator indicator
#[derive(Debug)]
pub struct GCodeInd<'a> {
    coord: Coord,
    pub gcode: GCode,
    index: usize,
    config: &'a Config,
}

impl <'a> GCodeInd<'a> {
    /// Create a new [`GCodeInd`]
    pub fn new(config: &'a Config) -> Self {
        Self {
            coord: Coord::default(),
            gcode: GCode::default(),
            index: 0,
            config,
        }
    }
}

impl <'a> Indicate for GCodeInd<'a> {
    /// G-code generator start
    fn initiation(&mut self) {
        // G-code header
        self.gcode.push(Line::from(&Void).label("Generated by ThdMaker Slicer"));
        self.gcode.push(Line::from(&Void).label(format!("Layer height: {}mm", self.config.layer_height)));
        self.gcode.push(Line::from(&Void).label(format!("Line width: {}mm", self.config.line_width)));
        self.gcode.push(Line::from(&Void).label(format!("Infill: {:.0}%", self.config.infill_density * 100.0)));
        self.gcode.push(Line::from(&Void));

        // Start G-code
        self.gcode.push(Line::from(&G21));
        self.gcode.push(Line::from(&G90));
        self.gcode.push(Line::from(&G28::default()));
        self.gcode.push(Line::from(&*M190::default().set_s(self.config.bed_temperature)));
        self.gcode.push(Line::from(&*M104::default().set_s(self.config.nozzle_temperature)));
        self.gcode.push(Line::from(&*G1::default().set_y(2.0).set_f(3000.0)).label("Move to safe height"));
        self.gcode.push(Line::from(&*G92::default().set_e(0.0)).label("Reset extruder"));
        self.gcode.push(Line::from(&Void));
    }

    /// Layer begin, move to layer height
    fn laybegin(&mut self, index: usize, _: usize) {
        self.index = index;
        self.gcode.push(Line::from(&Void).label(format!("Layer {}\n", self.index + 1)));
        self.gcode.push(Line::from(&*G1::default().set_z(self.config.height(self.index)).set_f(self.config.travel_speed)));
    }

    /// Draw a line segment: the head stands at its start, so the move prints it.
    fn drawline(&mut self, segment: &Segment2D) {
        // Print segment - map Point2D (x, z) to printer (X, Y)
        let speed = if self.index == 0 {
            self.config.first_speed
        } else {
            self.config.print_speed
        };
        self.gcode.push(Line::from(&*G1::default().set_x(segment.end.x).set_y(segment.end.z).set_e(self.coord.extruder).set_f(speed)));
    }

    /// Idle travel move, to the start of the segment
    fn idleride(&mut self, segment: &Segment2D) {
        self.gcode.push(Line::from(&*G0::default().set_x(segment.start.x).set_y(segment.start.z).set_f(self.config.travel_speed)));
    }

    /// Extrusion
    fn extrusion(&mut self) {
        self.gcode.push(Line::from(&*G1::default().set_e(self.coord.extruder).set_f(self.config.retraction_speed)));
    }

    /// Retraction
    fn retraction(&mut self) {
        self.gcode.push(Line::from(&*G1::default().set_e(self.coord.extruder).set_f(self.config.retraction_speed)));
    }

    /// Layer finish, retract and move to next layer height
    fn layfinish(&mut self) {
        self.gcode.push(Line::from(&Void));
    }

    /// Completion, move to safe height and home
    fn completion(&mut self) {
        // End G-code
        self.gcode.push(Line::from(&Void).label("End of print"));
        self.gcode.push(Line::from(&*G1::default().set_y(10.0).set_f(3000.0)).label("Move to safe height"));
        self.gcode.push(Line::from(&*G28::default().set_x(0.0).set_y(0.0)));
        self.gcode.push(Line::from(&*M104::default().set_s(0.0)));
        self.gcode.push(Line::from(&*M140::default().set_s(0.0)));
        self.gcode.push(Line::from(&M84));
    }

    /// Current position
    fn coord(&mut self) -> &mut Coord {
        &mut self.coord
    }

    /// Config
    fn config(&self) -> &Config {
        self.config
    }

    /// Current layer index
    fn index(&self) -> usize {
        self.index
    }
}

/// Strip line draw indicator
///
/// Produces the preview payload: positions plus linear RGBA vertex
/// colours, so the viewer needs no colour decision of its own.
pub struct StripInd<'a> {
    coord: Coord,
    index: usize,
    height: f32,
    pub positions: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
    solid: [f32; 4],
    empty: [f32; 4],
    config: &'a Config,
}

impl <'a> StripInd<'a> {
    /// Create a new [`StripInd`]
    pub fn new(config: &'a Config) -> Self {
        Self {
            coord: Coord::default(),
            index: 0,
            height: 0.0,
            positions: Vec::new(),
            colors: Vec::new(),
            // Colors for different line types
            solid: [0.0, 1.0, 0.0, 1.0],
            empty: [0.4, 0.5, 0.6, 1.0],

            config,
        }
    }

    /// Append a vertex at the current layer height.
    ///
    /// A line strip cannot change colour in the middle of a leg, so the previous
    /// vertex is repeated when the colour changes: the repeated pair has no
    /// length and every leg ends up with a single colour.
    fn push(&mut self, x: f32, z: f32, color: [f32; 4]) {
        let lastpos = self.positions.last().copied();
        let lastclr = self.colors.last().copied();
        if let (Some(previous), Some(last)) = (lastpos, lastclr) {
            if last != color {
                self.positions.push(previous);
                self.colors.push(color);
            }
        }

        self.positions.push([x, self.height, z]);
        self.colors.push(color);
    }
}

impl <'a> Indicate for StripInd<'a> {
    /// Strip start
    fn initiation(&mut self) {
        // The head starts at the origin, idle.
        self.push(0.0, 0.0, self.empty);
    }

    /// Layer begin, move to layer height
    fn laybegin(&mut self, index: usize, _: usize) {
        self.index = index;
        self.height = self.config.height(self.index);

        // The head rises where it stands, so the ride into this layer starts from
        // the height it was at.
        if let Some(last) = self.positions.last() {
            self.push(last[0], last[2], self.empty);
        }
    }

    /// Draw a line segment: the head stands at its start, so the leg prints it.
    fn drawline(&mut self, segment: &Segment2D) {
        self.push(segment.start.x, segment.start.z, self.solid);
        self.push(segment.end.x, segment.end.z, self.solid);
    }

    /// Idle travel move, to the start of the segment
    fn idleride(&mut self, segment: &Segment2D) {
        self.push(segment.start.x, segment.start.z, self.empty);
    }

    /// Extrusion
    fn extrusion(&mut self) {
    }

    /// Retraction
    fn retraction(&mut self) {
    }

    /// Layer finish, retract and move to next layer height
    fn layfinish(&mut self) {
    }

    /// Completion, move to safe height and home
    fn completion(&mut self) {
    }

    /// Current position
    fn coord(&mut self) -> &mut Coord {
        &mut self.coord
    }

    /// Config
    fn config(&self) -> &Config {
        self.config
    }

    /// Current layer index
    fn index(&self) -> usize {
        self.index
    }
}
