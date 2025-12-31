use std::fs::File;
use std::io::{BufReader, BufRead, Cursor, Read, Seek, SeekFrom};
use std::path::Path;
use byteorder::{LittleEndian, ReadBytesExt};
use super::error::{Error, Result};
use super::define::*;

/// Read a STL file from a path.
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<Vec<Mesh>> {
    let file = File::open(path)?;
    read(file)
}

/// Read a STL file from bytes.
pub fn read_bytes(data: &[u8]) -> Result<Vec<Mesh>> {
    let cursor = Cursor::new(data);
    read(cursor)
}

/// Read a STL file from a reader.
pub fn read<R: Read + Seek>(reader: R) -> Result<Vec<Mesh>> {
    let mut reader = BufReader::new(reader);

    let mut format = Format::Binary;

    let mut probe = [0u8; 84]; // header size and face count
    let readed = reader.read(&mut probe)?;
    let probe = &probe[..readed];

    if probe.len() < 84 {
        format = Format::Ascii;
    } else {
        let solid = [115, 111, 108, 105, 100]; // 's', 'o', 'l', 'i', 'd'
        if probe[0..solid.len()] == solid[..] {
            format = Format::Ascii;
        }
    }

    reader.seek(SeekFrom::Start(0))?;

    match format {
        Format::Ascii => read_ascii(&mut reader),
        Format::Binary => read_binary(&mut reader),
    }
}

/// Read a STL file in ASCII format.
/// 
/// ASCII example:
/// ```
/// let ascii = 
/// b"solid foo bar
///       facet normal 0.1 0.2 0.3
///           outer loop
///               vertex 1 2 3
///               vertex 4 5 6e-15
///               vertex 7 8 9.87654321
///           endloop
///       endfacet
///   endsolid foo bar";
/// println!("{:?}", ascii);
/// ```
pub fn read_ascii<R: BufRead>(reader: &mut R) -> Result<Vec<Mesh>> {
    let mut lines = reader.lines();
    let mut meshs = Vec::new();
    let mut meshi = 0;
    let mut verti = 0;
    fn extract(line: Option<std::io::Result<String>>) -> Result<String> {
        line.ok_or(Error::UnexpectedError("line is none".to_owned()))?
            .map_err(Error::from)
            .map(|v| v.trim().to_string())
    }
    fn predict(line: String, expected: &str) -> Result<String> {
        if line.starts_with(expected) {
            Ok(line)
        } else {
            Err(Error::InvalidSegment{ expected: expected.to_owned(), received: line.to_owned() })
        }
    }
    while let Some(line) = lines.next() {
        let line = predict(line?.trim().to_string(), "solid")?;
        let mut mesh = Mesh::default();
        if line.len() > 5 {
            mesh.name = Some(line[5..].trim().to_string());
        }
        mesh.start = verti;
        meshs.push(mesh);

        while let Some(line) = lines.next() {
            let line = line?.trim().to_string();
            if line.starts_with("facet normal") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                // facet normal nx ny nz
                if parts.len() >= 5 {
                    let nx = parts[2].parse::<f32>()?;
                    let ny = parts[3].parse::<f32>()?;
                    let nz = parts[4].parse::<f32>()?;
                    meshs[meshi].normals.push(Vertex::from(&[nx, ny, nz]));
                }
    
                let _ = predict(extract(lines.next())?, "outer loop")?;
    
                for _ in 0..3 {
                    let line = predict(extract(lines.next())?, "vertex")?;
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    let mut vertex = Vertex::from(&[0.0, 0.0, 0.0]);
                    // vertex x y z
                    if parts.len() >= 4 {
                        vertex.x = parts[1].parse::<f32>()?;
                        vertex.y = parts[2].parse::<f32>()?;
                        vertex.z = parts[3].parse::<f32>()?;
                    }
                    meshs[meshi].vertices.push(vertex);
                    verti += 1;
                }

                let _ = predict(extract(lines.next())?, "endloop")?;
                let _ = predict(extract(lines.next())?, "endfacet")?;

                let idxlen = meshs[meshi].indices.len();
                meshs[meshi].indices.push([idxlen - 3, idxlen - 2, idxlen - 1]);
            } else {
                let _ = predict(extract(lines.next())?, "endsolid")?;
                meshi += 1;
                break;
            }
        }
    }
    Ok(meshs)
}

/// Read a STL file in binary format.
pub fn read_binary<R: BufRead>(reader: &mut R) -> Result<Vec<Mesh>> {
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    let mut cursor = Cursor::new(buffer);
    // read 80 bytes header
    let mut header = [0u8; 80];
    cursor.read_exact(&mut header)?;
    // face count
    let faces = cursor.read_u32::<LittleEndian>()?;
    
    let mut mesh = Mesh::default();

    // process STL header
    // check for default color in header ("COLOR=rgba" sequence).
    // it is 10 bytes long
    for i in 0..80-10 {
        let signi = i + 6;
        if header[i..signi] == *b"COLOR=" {
            // read 4 bytes rgba
            let color = Color::from(&[
                header[signi], header[signi+1],
                header[signi+2], header[signi+3],
            ]);
            mesh.color = Some(color);
            mesh.colors = Some(vec![color; faces as usize]);
            break;
        }
    }
    mesh.header = Some(header);
    mesh.attrs = Some(vec![0u16; faces as usize]);

    for f in 0..faces {
        let nx = cursor.read_f32::<LittleEndian>()?;
        let ny = cursor.read_f32::<LittleEndian>()?;
        let nz = cursor.read_f32::<LittleEndian>()?;
        let normal = Vertex::from(&[nx, ny, nz]);
        mesh.normals.push(normal);
        for _ in 0..3 {
            let vx = cursor.read_f32::<LittleEndian>()?;
            let vy = cursor.read_f32::<LittleEndian>()?;
            let vz = cursor.read_f32::<LittleEndian>()?;
            let v = Vertex::from(&[vx, vy, vz]);
            mesh.vertices.push(v);
        }
        let idxlen = mesh.indices.len();
        mesh.indices.push([idxlen - 3, idxlen - 2, idxlen - 1]);
        // read attribute byte count (Usually 0)
        let attr = cursor.read_u16::<LittleEndian>()?;
        if attr & 0x8000 == 0 {
            if let Some(colors) = mesh.colors.as_mut() {
                let mut color = colors[f as usize];
                color.r = (attr & 0x1F) as u8;
                color.g = ((attr >> 5) & 0x1F) as u8;
                color.b = ((attr >> 10) & 0x1F) as u8;
            }
        }
        if let Some(attrs) = mesh.attrs.as_mut() {
            attrs[f as usize] = attr;
        }
    }

    Ok(vec![mesh])
}
