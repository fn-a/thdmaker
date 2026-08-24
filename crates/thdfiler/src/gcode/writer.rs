use std::fs::File;
use std::io::{Cursor, Write, BufWriter};
use std::path::Path;
use super::error::Result;
use super::define::*;

impl GCode {
    pub fn write<W: Write>(&self, writer: W) -> Result<()> {
        let mut writer = BufWriter::new(writer);
        for line in &self.lines {
            line.write(&mut writer)?;
        }
        Ok(())
    }
}

impl Line {
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Line number
        if let Some(n) = self.linenum {
            write!(writer, "N{} ", n)?;
        }

        // Command
        match &self.command {
            Command::GCode(v) => {
                write!(writer, "G{v}")?;
            }
            Command::MCode(v) => {
                write!(writer, "M{v}")?;
            }
            Command::Other(w, v) => {
                write!(writer, "{w}{v}")?;
            }
            Command::None => {}
        }

        // Parameters
        for param in &self.parameters {
            write!(writer, " ")?;
            param.write(writer)?;
        }

        // Comment
        if let Some(comment) = &self.comment {
            if self.command != Command::None || self.linenum.is_some() {
                write!(writer, " ")?;
            }
            let comments = comment.split("\n").collect::<Vec<_>>();
            if comments.len() > 1 {
                writeln!(writer, "(")?;
                for line in comments {
                    writeln!(writer, "{}", line)?;
                }
                write!(writer, ")")?;
            } else {
                write!(writer, ";{}", comment)?;
            }
        }

        writeln!(writer)?;
        Ok(())
    }
}

impl Word {
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        if let Some(v) = self.value {
            write!(writer, "{}{}", self.letter, v)?;
        } else {
            write!(writer, "{}", self.letter)?;
        }
        Ok(())
    }
}

/// Write G-code to a file.
pub fn write_file<P: AsRef<Path>>(path: P, gcode: &GCode) -> Result<()> {
    let file = File::create(path)?;
    gcode.write(file)?;
    Ok(())
}

/// Write G-code to a byte vector.
pub fn write_bytes(gcode: &GCode) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    gcode.write(Cursor::new(&mut buffer))?;
    Ok(buffer)
}
