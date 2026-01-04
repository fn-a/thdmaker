/// Represents a G-code word (e.g., G1, X10.5).
#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    /// The letter (e.g., 'G', 'M', 'X').
    pub letter: char,
    /// The value associated with the letter.
    pub value: f32,
    /// The length of the word, including the letter and value.
    pub length: usize,
}

impl Default for Word {
    fn default() -> Self {
        Self {
            letter: 'G',
            value: 0.0,
            length: 2,
        }
    }
}

/// Represents a G-code command (e.g., G1, M3).
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// G-code command (e.g., G1, G2).
    GCode(Word),
    /// M-code command (e.g., M3, M5).
    MCode(Word),
    /// Other command (e.g., S100, F500).
    Other(Word),
    /// No command (e.g., only comment line).
    None,
}

impl Default for Command {
    fn default() -> Self {
        Self::None
    }
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

/// Represents G-code file, a collection of G-code lines.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GCode {
    /// List of G-code lines.
    pub lines: Vec<Line>,
}