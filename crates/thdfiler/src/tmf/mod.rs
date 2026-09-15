//! 3MF Specifications
//!
//!  Specification | ISO/IEC 25422 | ISO PDF | Latest Rev. PDF | Github | Description | Updated
//! ---|---|---|---|---|---|---
//!  3MF Core Specification | v1.3.0 | v1.3.0 | v1.4.0 | Github | Core 3MF Mesh Specification | 02/27/2025
//!  Beam Lattice Extension | v1.2.0 | v1.2.0 |  | Github | Beam lattice structures as graph of nodes and beam diameter | 02/27/2025
//!  Boolean Operations Extension | v1.1.0 | v1.1.0 | v1.1.1 | Github | Boolean operations for shape modification | 04/03/2025
//!  Displacement Extension  |  |  | v1.0.0<br> | Github | 3D Textured meshes by modifying mesh surface with displacement mapping | 02/27/2025
//!  Materials and Properties Extension | v1.2.1 | v1.2.1 |  | Github | Full color and multi-material definitions | 02/27/2025
//!  Production Extension | v1.1.2 | v1.1.2 |  | Github | Non-object resources &amp; attributes to the build section for uniquely identifying parts within a particular 3MF package | 02/27/2025
//!  Secure Content Extension | v1.0.2 | v1.0.2 |  | Github | Encryption mechanism to protect the 3MF content files | 02/27/2025
//!  Slice Extension | v1.0.2 | v1.0.2 |  | Github | 2D sliced model data in a 3MF package | 02/27/2025
//!  Volumetric Extension |  |  | v0.8.0 | Github | Support for voxel and/or implicit data within 3mf files | 02/27/2025

pub mod error;
pub mod define;
pub mod reader;
pub mod writer;

#[cfg(feature = "bevy")]
pub mod compat;
