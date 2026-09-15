# thdslicer

English | [中文](README.zh.md)

A 3D printing slicing library: slicing a 3D mesh into layers, generating infill paths and G-code.

## Module Structure

- `geometry`: Basic 2D geometry types (points, segments, etc.)
- `layer`: Contour building and processing of sliced layers
- `infill`: Infill patterns (dense / sparse infill)
- `indicate`: Output indicator abstraction, supporting `tris` previews or G-code generation

Both the input and the preview output use the `tris` intermediate format of
[`thdfiler`](../thdfiler/README.md).