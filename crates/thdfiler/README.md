# thdfiler

English | [中文](README.zh.md)

A 3D manufacturing file format library: reading and writing AMF, STL, 3MF, GCode and other formats.

## Module Structure

- `tmf`: 3MF format (including beam lattice, boolean operations, displacement, material, production, secure content and slice body extensions)
- `amf`: AMF format
- `stl`: STL format (binary and ASCII)
- `gcode`: GCode format
- `util`: Common utilities such as path handling

Each format module is organized as `define` / `reader` / `writer` / `error`.