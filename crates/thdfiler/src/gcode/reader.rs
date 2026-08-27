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
                if let Some(comment) = lstr.strip_prefix('(') {
                    if comment.is_empty() {
                        continue;
                    } else {
                        if let Some(inner) = comment.strip_suffix(')') {
                            lines.push(Line {
                                comment: Some(inner.to_string()),
                                ..Default::default()
                            });
                        } else {
                            comments.push(comment.to_string());
                        }
                    }
                } else if let Some(comment) = lstr.strip_prefix(';') {
                    lines.push(Line {
                        comment: Some(comment.to_string()),
                        ..Default::default()
                    });
                } else {
                    lines.push(Line::parse(lstr)?);
                }
            } else {
                if lstr.ends_with(")") {
                    lines.push(Line {
                        comment: Some(comments.join("\n")),
                        ..Default::default()
                    });
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
    fn produce(&self, line: &mut Line, keep: &[char]) -> Result<()> {
        let word = keep.iter().collect::<String>();
        let letter = keep[0];
        let number = if word.len() > 1 { &word[1..] } else { "" };
        match self {
            Pattern::Linenum => {
                line.linenum = Some(number.parse()?);
            }
            Pattern::Command => {
                line.command = match letter {
                    'G' => {
                        Command::GCode(number.parse()?)
                    }
                    'M' => {
                        Command::MCode(number.parse()?)
                    }
                    _ => {
                        Command::Other(letter, number.parse()?)
                    }
                }
            }
            Pattern::Parameter => {
                line.parameters.push(Word { letter, value: if number.is_empty() { None } else { Some(number.parse()?) } });
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