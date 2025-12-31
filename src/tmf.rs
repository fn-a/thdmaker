/// 3MF Specifications
/// 
///  Specification | ISO/IEC 25422 | ISO PDF | Latest Rev. PDF | Github | Description | Updated 
/// ---|---|---|---|---|---|---
///  3MF Core Specification | v1.3.0 | v1.3.0 | v1.4.0 | Github | Core 3MF Mesh Specification | 02/27/2025 
///  Beam Lattice Extension | v1.2.0 | v1.2.0 |  | Github | Beam lattice structures as graph of nodes and beam diameter | 02/27/2025 
///  Boolean Operations Extension | v1.1.0 | v1.1.0 | v1.1.1 | Github | Boolean operations for shape modification | 04/03/2025 
///  Displacement Extension  |  |  | v1.0.0<br> | Github | 3D Textured meshes by modifying mesh surface with displacement mapping | 02/27/2025 
///  Materials and Properties Extension | v1.2.1 | v1.2.1 |  | Github | Full color and multi-material definitions | 02/27/2025 
///  Production Extension | v1.1.2 | v1.1.2 |  | Github | Non-object resources &amp; attributes to the build section for uniquely identifying parts within a particular 3MF package | 02/27/2025 
///  Secure Content Extension | v1.0.2 | v1.0.2 |  | Github | Encryption mechanism to protect the 3MF content files | 02/27/2025 
///  Slice Extension | v1.0.2 | v1.0.2 |  | Github | 2D sliced model data in a 3MF package | 02/27/2025 
///  Volumetric Extension |  |  | v0.8.0 | Github | Support for voxel and/or implicit data within 3mf files | 02/27/2025 

pub mod error;
pub mod define;
pub mod reader;
pub mod writer;

pub mod path {
    use std::path::{Path, PathBuf, Component};
    /// Normalize a path by removing leading slash for ZIP.
    pub fn normalize(path: &str, dir: Option<&str>) -> String {
        absolute(path, dir).strip_prefix('/').unwrap_or(path).to_string()
    }

    /// Resolve a path relative to a directory.
    pub fn resolve(path: &str, dir: Option<&str>) -> String {
        let base = Path::new(dir.unwrap_or("/"));
        let relative = Path::new(path);
        
        let combined = base.join(relative);
        
        let components = combined.components().peekable();
        let mut result = PathBuf::new();

        for component in components {
            match component {
                Component::Normal(_) | 
                Component::RootDir |
                Component::CurDir => {
                    result.push(component);
                },
                Component::ParentDir => {
                    if !result.pop() {
                        result.push(component);
                    }
                },
                Component::Prefix(prefix) => {
                    result.push(prefix.as_os_str());
                }
            }
        }
        
        result.to_str().unwrap_or(path).to_string()
    }

    /// Get the absolute path with the directory.
    pub fn absolute(path: &str, dir: Option<&str>) -> String {
        if path.starts_with('/') {
            path.to_string()
        } else if path.starts_with(".") || dir.is_some() {
            // relative path to absolute path
            // e.g. path: "../../file" dir: "/base/dir" -> "/base/dir/file"
            let base_dir = Path::new(dir.unwrap_or("/"));
            let relative = Path::new(path);
            let mut result = base_dir.to_path_buf();
            for component in relative.components() {
                match component {
                    Component::Normal(name) => {
                        result.push(name);
                    },
                    // ignore .. and .
                    Component::ParentDir | Component::CurDir => {
                        continue;
                    },
                    _ => {
                        result.push(component);
                    }
                }
            }
            result.to_str().unwrap_or(path).to_string()
        } else {
            "/".to_owned() + path
        }
    }

    /// Get the extension of a path.
    pub fn extension(path: &str) -> String {
        Path::new(path).extension().and_then(|ext| ext.to_str()).unwrap_or_default().to_string()
    }
}