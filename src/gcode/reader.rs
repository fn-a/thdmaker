use std::path::Path;
use std::fs::File;
use std::io::{BufReader, Read};
use super::error::{Error, Result};
use super::define::*;

impl GCode {
    fn parse(input: &str) -> Result<Self> {
        let mut lines = Vec::new();
        let mut comments = Vec::new();
        for line in input.lines() {
            let lstr = line.trim();
            if lstr.is_empty() || lstr == "%" {
                continue;
            }
            if comments.is_empty() {
                if lstr.starts_with('(') {
                    let comment = lstr[1..lstr.len()].to_string();
                    if comment.is_empty() {
                        continue;
                    } else {
                        if comment.ends_with(')') {
                            let mut line = Line::default();
                            line.comment = Some(comment[..comment.len() - 1].to_string());
                            lines.push(line);
                        } else {
                            comments.push(comment);
                        }
                    }
                } else if lstr.starts_with(';') {
                    let mut line = Line::default();
                    line.comment = Some(lstr[1..].to_string());
                    lines.push(line);
                } else {
                    lines.push(Line::parse(lstr)?);
                }
            } else {
                if lstr.ends_with(")") {
                    let comment = comments.join("\n");
                    let mut line = Line::default();
                    line.comment = Some(comment);
                    lines.push(line);
                    comments.clear();
                } else {
                    comments.push(lstr.to_string());
                }
            }
        }
        Ok(GCode { lines })
    }
}

#[derive(PartialEq, PartialOrd)]
enum Pattern {
    Initial,
    Linenum,
    Command,
    Parameter,
}

impl Pattern {
    fn produce(&self, line: &mut Line, keep: &Vec<char>) -> Result<()> {
        let word = keep.iter().collect::<String>();
        let length = word.len();
        let letter = keep[0];
        let number = if word.len() > 1 { word[1..].to_string() } else { "0".to_string() };
        match self {
            Pattern::Linenum => {
                line.linenum = Some(number.parse()?);
            }
            Pattern::Command => {
                let word = Word { letter, value: number.parse()?, length };
                line.command = match letter {
                    'G' => {
                        Command::GCode(word)
                    }
                    'M' => {
                        Command::MCode(word)
                    }
                    _ => {
                        Command::Other(word)
                    }
                }
            }
            Pattern::Parameter => {
                line.parameters.push(Word { letter, value: number.parse()?, length });
            }
            Pattern::Initial => {}
        };
        Ok(())
    }
}

impl Line {
    fn parse(input: &str) -> Result<Self> {
        let mut line = Line::default();
        let mut keep = Vec::new();
        let mut pattern = Pattern::Initial;
        for (i, c) in input.chars().enumerate() {
            if c.is_whitespace() {
                continue;
            }
            if c.is_ascii_alphabetic() {
                if !keep.is_empty() {
                    pattern.produce(&mut line, &keep)?;
                    keep.clear();
                }
                let letter = c.to_ascii_uppercase();
                keep.push(letter);
                match letter {
                    'N' if pattern == Pattern::Initial => { pattern = Pattern::Linenum; }
                    _ => {
                        pattern = match pattern {
                            Pattern::Initial | Pattern::Linenum => Pattern::Command,
                            _ => Pattern::Parameter,
                        };
                    }
                }
            } else {
                match c {
                    '-' | '.' | '+' | '0'..='9' => {
                        keep.push(c);
                    }
                    ';' => {
                        // M104 S0 ;extruder heater off
                        line.comment = Some(input[(i + 1)..].to_string());
                        break;
                    }
                    '(' => {
                        // G01 X-40 (Position 1)
                        line.comment = Some(input[(i + 1)..(input.len() - 1)].to_string());
                        break;
                    }
                    _ => return Err(Error::InvalidCharacter(c))
                }
            }
        }
        if !keep.is_empty() {
            pattern.produce(&mut line, &keep)?;
        }
        Ok(line)
    }
}

/// Read G-code from a file.
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<GCode> {
    let file = File::open(path.as_ref())?;
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    GCode::parse(&content)
}

/// Read G-code from bytes.
pub fn read_bytes(data: &[u8]) -> Result<GCode> {
    GCode::parse(str::from_utf8(data)?)
}