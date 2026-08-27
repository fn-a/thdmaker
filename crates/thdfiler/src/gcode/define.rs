use std::f32::consts;

/// Represents a G-code word (e.g., G1, X10.5).
#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    /// The letter (e.g., 'G', 'M', 'X').
    pub letter: char,
    /// The value associated with the letter.
    pub value: Option<f32>,
}

/// Represents a G-code command (e.g., G1, M3).
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Command {
    /// G-code command (e.g., G1, G2).
    GCode(usize),
    /// M-code command (e.g., M3, M5).
    MCode(usize),
    /// Other command (e.g., S100, F500).
    Other(char, usize),
    /// No command (e.g., only comment line).
    #[default]
    None,
}


/// Represents a parsed line of G-code.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Line {
    /// G-code command.
    pub command: Command,
    /// List of parameters in the line.
    pub parameters: Vec<Word>,
    /// Line number (N value), if present.
    pub linenum: Option<u32>,
    /// Comment, if present.
    pub comment: Option<String>,
}

impl Line {
    pub fn route(&self) -> Box<dyn Route> {
        match self.command {
            Command::GCode(v) => match v {
                0 => Box::new(G0::from(&self.parameters)),
                1 => Box::new(G1::from(&self.parameters)),
                2 => Box::new(G2::from(&self.parameters)),
                3 => Box::new(G3::from(&self.parameters)),
                90 => Box::new(G90),
                91 => Box::new(G91),
                _ => Box::new(Void),
            },
            Command::MCode(v) => match v {
                82 => Box::new(M82),
                83 => Box::new(M83),
                _ => Box::new(Void),
            },
            _ => Box::new(Void),
        }
    }

    pub fn label(mut self, txt: impl Into<String>) -> Self {
        self.comment = Some(txt.into());
        self
    }

}

/// Represents G-code file, a collection of G-code lines.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GCode {
    /// List of G-code lines.
    pub lines: Vec<Line>,
}

impl GCode {
    pub fn push(&mut self, line: Line) {
        self.lines.push(line);
    }
}

/// Represents a Nozzle
#[derive(Debug, Clone, PartialEq)]
pub struct Arrow {
    /// The current position of the nozzle.
    pub position: [f32; 3],
    /// Whether the position is absolute or relative.
    pub absolute: bool,
    /// Whether the extruder is absolute or relative and the current extruder value.
    pub extruder: (bool, f32),
}

impl Default for Arrow {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            absolute: true,
            extruder: (true, 0.0),
        }
    }
}

/// Represents points and the move type of the nozzle.
#[derive(Debug, Clone, PartialEq)]
pub struct March {
    pub points: Vec<[f32; 3]>,
    pub solid: bool,
}

impl March {
    pub fn get_positions(&self) -> Vec<[f32; 3]> {
        self.points.iter().map(|v| {
            // Convert Z-up coordinates to Y-up coordinates
            // Z-up: (x, y, z) -> Y-up: (x, z, -y)
            [v[0], v[2], -v[1]]
        }).collect()
    }
}

pub trait Route {
    fn march(&self, _: &mut Arrow) -> March {
        March {
            points: vec![],
            solid: false,
        }
    }
}

/// Represents a void route, which does not move the nozzle.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Void;

impl Route for Void {}

impl From<&Void> for Line {
    fn from(_: &Void) -> Self {
        Self::default()
    }
}

/// Rapid Move.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct G0 {
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>,
    /// The feed rate to use for the move.
    f: Option<f32>,
}

impl G0 {
    pub fn set_x(&mut self, x: f32) -> &mut Self {
        self.x = Some(x);
        self
    }

    pub fn set_y(&mut self, y: f32) -> &mut Self {
        self.y = Some(y);
        self
    }

    pub fn set_z(&mut self, z: f32) -> &mut Self {
        self.z = Some(z);
        self
    }

    pub fn set_f(&mut self, f: f32) -> &mut Self {
        self.f = Some(f);
        self
    }
}

impl Route for G0 {
    fn march(&self, arrow: &mut Arrow) -> March {
        let mut position = arrow.position;
        if let Some(x) = self.x {
            position[0] = if arrow.absolute { x } else { position[0] + x };
        }
        if let Some(y) = self.y {
            position[1] = if arrow.absolute { y } else { position[1] + y };
        }
        if let Some(z) = self.z {
            position[2] = if arrow.absolute { z } else { position[2] + z };
        }
        arrow.position = position;
        March {
            
            points: vec![position],
            solid: false,
        }
    }
}

impl From<&Vec<Word>> for G0 {
    fn from(words: &Vec<Word>) -> Self {
        let mut g0 = Self::default();
        for word in words {
            match word.letter {
                'X' => g0.x = word.value,
                'Y' => g0.y = word.value,
                'Z' => g0.z = word.value,
                'F' => g0.f = word.value,
                _ => {}
            }
        }
        g0
    }
}

impl From<&G0> for Line {
    fn from(g0: &G0) -> Self {
        let mut words = vec![];
        let params = ['X', 'Y', 'Z', 'F'];
        let values = [g0.x, g0.y, g0.z, g0.f];
        for (i, &value) in values.iter().enumerate() {
            words.push(Word { letter: params[i], value });
        }
        Line {
            command: Command::GCode(0),
            parameters: words,
            linenum: None,
            comment: None,
        }
    }
}

/// Linear Move.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct G1 {
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>,
    /// The amount of extrusion to perform.
    e: Option<f32>,
    /// The feed rate to use for the move.
    f: Option<f32>,
}

impl G1 {
    pub fn set_x(&mut self, x: f32) -> &mut Self {
        self.x = Some(x);
        self
    }

    pub fn set_y(&mut self, y: f32) -> &mut Self {
        self.y = Some(y);
        self
    }

    pub fn set_z(&mut self, z: f32) -> &mut Self {
        self.z = Some(z);
        self
    }

    pub fn set_e(&mut self, e: f32) -> &mut Self {
        self.e = Some(e);
        self
    }

    pub fn set_f(&mut self, f: f32) -> &mut Self {
        self.f = Some(f);
        self
    }
}

impl Route for G1 {
    fn march(&self, arrow: &mut Arrow) -> March {
        let mut position = arrow.position;
        if let Some(x) = self.x {
            position[0] = if arrow.absolute { x } else { position[0] + x };
        }
        if let Some(y) = self.y {
            position[1] = if arrow.absolute { y } else { position[1] + y };
        }
        if let Some(z) = self.z {
            position[2] = if arrow.absolute { z } else { position[2] + z };
        }
        let mut solid = true;
        if let Some(e) = self.e {
            arrow.extruder.1 = if arrow.extruder.0 { e } else { arrow.extruder.1 + e };
        } else {
            solid = false;
        }
        arrow.position = position;
        March {
            points: vec![position],
            solid,
        }
    }
}

impl From<&Vec<Word>> for G1 {
    fn from(words: &Vec<Word>) -> Self {
        let mut g1 = Self::default();
        for word in words {
            match word.letter {
                'X' => g1.x = word.value,
                'Y' => g1.y = word.value,
                'Z' => g1.z = word.value,
                'E' => g1.e = word.value,
                'F' => g1.f = word.value,
                _ => {}
            }
        }
        g1
    }
}

impl From<&G1> for Line {
    fn from(g1: &G1) -> Self {
        let mut words = vec![];
        let params = ['X', 'Y', 'Z', 'E', 'F'];
        let values = [g1.x, g1.y, g1.z, g1.e, g1.f];
        for (i, &value) in values.iter().enumerate() {
            words.push(Word { letter: params[i], value });
        }
        Line {
            command: Command::GCode(1),
            parameters: words,
            linenum: None,
            comment: None,
        }
    }
}

/// Arc Move.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Arc {
    /// The position to move to on the X axis.
    x: Option<f32>,
    /// The position to move to on the Y axis.
    y: Option<f32>,
    /// Optional position to move to on the Z axis.
    z: Option<f32>,
    /// The X offset of the arc center from the current position.
    i: Option<f32>,
    /// The Y offset of the arc center from the current position.
    j: Option<f32>,
    /// Optional Z offset of the arc center from the current position.
    k: Option<f32>,
    /// The amount of extrusion to perform.
    e: Option<f32>,
    /// The feed rate to use for the move.
    f: Option<f32>,
}

impl Arc {
    fn points(&self, arrow: &Arrow, direct: f32) -> Vec<[f32; 3]> {
        // Starting position [x, y, z]
        let start = arrow.position;
        // Ending position [x, y, z]
        let mut endpt = start;
        
        if let Some(x) = self.x {
            endpt[0] = if arrow.absolute { x } else { start[0] + x };
        }
        if let Some(y) = self.y {
            endpt[1] = if arrow.absolute { y } else { start[1] + y };
        }
        if let Some(z) = self.z {
            endpt[2] = if arrow.absolute { z } else { start[2] + z };
        }
        let i = self.i.unwrap_or(0.0);
        let j = self.j.unwrap_or(0.0);
        let k = self.k.unwrap_or(0.0);
        
        // Center of the arc [x, y, z]
        let center = [start[0] + i, start[1] + j, start[2] + k];
        // Calculate radius and angles in the XY plane
        let dx_start = start[0] - center[0];
        let dy_start = start[1] - center[1];
        let dx_endpt = endpt[0] - center[0];
        let dy_endpt = endpt[1] - center[1];
        
        let radius = (dx_start * dx_start + dy_start * dy_start).sqrt();
        
        // Avoid division by zero
        if radius < 1e-6 {
            vec![endpt]
        } else {
            // Calculate start and end angles
            let angle_start = dy_start.atan2(dx_start);
            let angle_endpt = dy_endpt.atan2(dx_endpt);
            
            // Calculate angular difference
            let mut angle_diff = angle_endpt - angle_start;
            
            // Normalize angle difference to [0, 2π]
            if angle_diff > 0.0 {
                angle_diff += direct * (2.0 * consts::PI);
            }
            
            // If angle difference is very small, just return the end point
            if angle_diff.abs() < 1e-6 {
                vec![endpt]
            } else {
                // Calculate number of segments based on arc length
                // Use a segment length of approximately 0.5mm
                // Limit to reasonable range
                let segments = ((radius * angle_diff.abs() / 0.5).ceil() as usize).clamp(1, 1000);
                
                let mut points = Vec::with_capacity(segments);
                
                // Generate points along the arc
                for i in 1..=segments {
                    let t = i as f32 / segments as f32;
                    let angle = angle_start + angle_diff * t;
                    
                    let x = center[0] + radius * angle.cos();
                    let y = center[1] + radius * angle.sin();
                    
                    // Linearly interpolate Z
                    let z = start[2] + (endpt[2] - start[2]) * t;
                    
                    points.push([x, y, z]);
                }
                
                points
            }
        }
    }
}

impl From<&Vec<Word>> for Arc {
    fn from(words: &Vec<Word>) -> Self {
        let mut arc = Self::default();
        for word in words {
            match word.letter {
                'X' => arc.x = word.value,
                'Y' => arc.y = word.value,
                'Z' => arc.z = word.value,
                'I' => arc.i = word.value,
                'J' => arc.j = word.value,
                'K' => arc.k = word.value,
                'E' => arc.e = word.value,
                'F' => arc.f = word.value,
                _ => {}
            }
        }
        arc
    }
}

/// Clockwise Arc Move.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct G2(Arc);

impl Route for G2 {
    fn march(&self, arrow: &mut Arrow) -> March {
        let points = self.0.points(arrow, -1.0);
        
        if let Some(e) = self.0.e {
            arrow.extruder.1 = if arrow.extruder.0 { e } else { arrow.extruder.1 + e };
        }
        
        if let Some(last) = points.last() {
            arrow.position = *last;
        }
        
        March {
            points,
            solid: true,
        }
    }
}

impl From<&Vec<Word>> for G2 {
    fn from(words: &Vec<Word>) -> Self {
        G2(Arc::from(words))
    }
}

/// Counter-Clockwise Arc Move.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct G3(Arc);

impl Route for G3 {
    fn march(&self, arrow: &mut Arrow) -> March {
        let points = self.0.points(arrow, 1.0);
        
        if let Some(e) = self.0.e {
            arrow.extruder.1 = if arrow.extruder.0 { e } else { arrow.extruder.1 + e };
        }
        
        if let Some(last) = points.last() {
            arrow.position = *last;
        }
        
        March {
            points,
            solid: true,
        }
    }
}

impl From<&Vec<Word>> for G3 {
    fn from(words: &Vec<Word>) -> Self {
        G3(Arc::from(words))
    }
}

/// Set units to millimeters.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct G21;

impl From<&G21> for Line {
    fn from(_: &G21) -> Self {
        Line {
            command: Command::GCode(21),
            parameters: vec![],
            linenum: None,
            comment: Some("Set units to millimeters".into()),
        }
    }
}

/// Move to Origin (Home).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct G28 {
    /// Flag to go back to the X axis origin.
    x: Option<f32>,
    /// Flag to go back to the Y axis origin.
    y: Option<f32>,
    /// Flag to go back to the Z axis origin.
    z: Option<f32>,
}

impl G28 {
    pub fn set_x(&mut self, x: f32) -> &mut Self {
        self.x = Some(x);
        self
    }

    pub fn set_y(&mut self, y: f32) -> &mut Self {
        self.y = Some(y);
        self
    }

    pub fn set_z(&mut self, z: f32) -> &mut Self {
        self.z = Some(z);
        self
    }
}

impl From<&G28> for Line {
    fn from(g28: &G28) -> Self {
        let mut texts = Vec::new();
        let mut words = Vec::new();
        let params = ['X', 'Y', 'Z'];
        let values = [g28.x, g28.y, g28.z];
        for (i, &value) in values.iter().enumerate() {
            let p = params[i];
            texts.push(value.map(|v| format!("{}{}", p, v)).unwrap_or(p.into()));
            words.push(Word { letter: p, value });
        }
        let mut label = "Home ".to_owned();
        if !texts.is_empty() {
            label.push_str(&texts.join(" and "));
        } else {
            label.push_str("all axes");
        }
        Line {
            command: Command::GCode(28),
            parameters: words,
            linenum: None,
            comment: Some(label),
        }
    }
}

/// Set to Absolute Positioning.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct G90;

impl From<&G90> for Line {
    fn from(_: &G90) -> Self {
        Line {
            command: Command::GCode(90),
            parameters: vec![],
            linenum: None,
            comment: Some("Use absolute coordinates".into()),
        }
    }
}

impl Route for G90 {
    fn march(&self, arrow: &mut Arrow) -> March {
        arrow.absolute = true;
        March {
            points: vec![],
            solid: false,
        }
    }
}

/// Set to Relative Positioning.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct G91;

impl Route for G91 {
    fn march(&self, arrow: &mut Arrow) -> March {
        arrow.absolute = false;
        March {
            points: vec![],
            solid: false,
        }
    }
}

/// Set Position.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct G92 {
    /// new X axis position.
    x: Option<f32>,
    /// new Y axis position.
    y: Option<f32>,
    /// new Z axis position.
    z: Option<f32>,
    /// new extruder position.
    e: Option<f32>,
}

impl G92 {
    pub fn set_x(&mut self, x: f32) -> &mut Self {
        self.x = Some(x);
        self
    }
    pub fn set_y(&mut self, y: f32) -> &mut Self {
        self.y = Some(y);
        self
    }
    pub fn set_z(&mut self, z: f32) -> &mut Self {
        self.z = Some(z);
        self
    }
    pub fn set_e(&mut self, e: f32) -> &mut Self {
        self.e = Some(e);
        self
    }
}

impl From<&G92> for Line {
    fn from(g92: &G92) -> Self {
        let mut words = vec![];
        let params = ['X', 'Y', 'Z', 'E'];
        let values = [g92.x, g92.y, g92.z, g92.e];
        for (i, &value) in values.iter().enumerate() {
            words.push(Word { letter: params[i], value });
        }
        Line {
            command: Command::GCode(92),
            parameters: words,
            linenum: None,
            comment: None,
        }
    }
}

/// Set extruder to absolute mode.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct M82;

impl Route for M82 {
    fn march(&self, arrow: &mut Arrow) -> March {
        arrow.extruder.0 = true;
        March {
            points: vec![],
            solid: false,
        }
    }
}

/// Set extruder to relative mode.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct M83;

impl Route for M83 {
    fn march(&self, arrow: &mut Arrow) -> March {
        arrow.extruder.0 = false;
        March {
            points: vec![],
            solid: false,
        }
    }
}

/// Stop the idle hold on all axis and extruder.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct M84;

impl From<&M84> for Line {
    fn from(_: &M84) -> Self {
        Line {
            command: Command::GCode(84),
            parameters: vec![],
            linenum: None,
            comment: Some("Disable motors".into()),
        }
    }
}

/// Set Extruder Temperature.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct M104 {
    /// Target temperature.
    s: Option<f32>,
}

impl M104 {
    pub fn set_s(&mut self, s: f32) -> &mut Self {
        self.s = Some(s);
        self
    }
}

impl From<&M104> for Line {
    fn from(m104: &M104) -> Self {
        Line {
            command: Command::GCode(104),
            parameters: vec![Word { letter: 'S', value: m104.s }],
            linenum: None,
            comment: if let Some(v) = m104.s {
                Some(format!("Set extruder temperature to {}C", v))
            } else {
                Some("Turn off extruder".into())
            },
        }
    }
}

/// Set Bed Temperature (Fast).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct M140 {
    /// Active/Target temperature.
    s: Option<f32>,
}

impl M140 {
    pub fn set_s(&mut self, s: f32) -> &mut Self {
        self.s = Some(s);
        self
    }
}

impl From<&M140> for Line {
    fn from(m140: &M140) -> Self {
        Line {
            command: Command::GCode(140),
            parameters: vec![Word { letter: 'S', value: m140.s }],
            linenum: None,
            comment: if let Some(v) = m140.s {
                Some(format!("Set bed temperature to {}C", v))
            } else {
                Some("Turn off bed".into())
            },
        }
    }
}

/// Wait for bed temperature to reach target temp.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct M190 {
    s: Option<f32>,
}

impl M190 {
    pub fn set_s(&mut self, s: f32) -> &mut Self {
        self.s = Some(s);
        self
    }
}

impl From<&M190> for Line {
    fn from(m190: &M190) -> Self {
        Line {
            command: Command::GCode(190),
            parameters: vec![Word { letter: 'S', value: m190.s }],
            linenum: None,
            comment: Some(format!("Wait for bed temperature to reach {}C", m190.s.unwrap_or_default())),
        }
    }
}