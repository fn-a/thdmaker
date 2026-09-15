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

pub mod math {
    use glam::{Mat4, Vec3};

    /// Whether `m` is (close to) the identity matrix.
    #[inline]
    pub fn identitied(m: &Mat4) -> bool {
        m.abs_diff_eq(Mat4::IDENTITY, 1e-6)
    }

    /// Transforms a point.
    ///
    /// The array form keeps the call sites (the slicer, the file bridges) free of any vector type.
    #[inline]
    pub fn transpoint(m: &Mat4, p: &[f32; 3]) -> [f32; 3] {
        m.transform_point3(Vec3::from_array(*p)).to_array()
    }

    /// Transforms a direction (no translation).
    #[inline]
    pub fn transdirect(m: &Mat4, d: &[f32; 3]) -> [f32; 3] {
        m.transform_vector3(Vec3::from_array(*d)).to_array()
    }

    /// Normal matrix: the inverse transpose, for transforming normals.
    ///
    /// Falls back to the input when the linear part is singular (a zero scale).
    #[inline]
    pub fn normatrix(m: &Mat4) -> Mat4 {
        let n = m.inverse().transpose();
        if n.is_finite() { n } else { *m }
    }

    /// Normalises a vector in place.
    #[inline]
    pub fn normalize(v: &mut [f32; 3]) {
        *v = Vec3::from_array(*v).normalize_or_zero().to_array();
    }

    // Z-up ⇄ Y-up

    /// The Z-up to Y-up axis swap `(x, y, z) -> (x, z, -y)`.
    pub const AXIS_SWAP: Mat4 = Mat4::from_cols_array(&[
        1.0, 0.0, 0.0, 0.0, //
        0.0, 0.0, -1.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 0.0, 1.0,
    ]);

    /// The inverse of [`AXIS_SWAP`], `(x, y, z) -> (x, -z, y)`.
    pub const AXIS_SWIV: Mat4 = Mat4::from_cols_array(&[
        1.0, 0.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        0.0, -1.0, 0.0, 0.0, //
        0.0, 0.0, 0.0, 1.0,
    ]);

    /// Re-expresses a Z-up matrix in the Y-up space the engine uses: `S · M · S⁻¹`.
    ///
    /// The file readers map positions with `(x, y, z) -> (x, z, -y)`; a matrix
    /// acting in the original Z-up space must be conjugated the same way so that
    /// `M · position` keeps working after the swap.
    #[inline]
    pub fn axis_ztoyup(m: &Mat4) -> Mat4 {
        AXIS_SWAP * *m * AXIS_SWIV
    }

    /// Re-expresses a Y-up matrix in the Z-up space the file formats use: `S⁻¹ · M · S`.
    ///
    /// This performs the inverse operation of axis Z-up to Y-up, converting
    /// a matrix acting in Y-up space back to the original Z-up space.
    #[inline]
    pub fn axis_ytozup(m: &Mat4) -> Mat4 {
        AXIS_SWIV * *m * AXIS_SWAP
    }

    /// A Z-up point in the Y-up space.
    #[inline]
    pub fn point_ztoyup(p: [f32; 3]) -> [f32; 3] {
        [p[0], p[2], -p[1]]
    }

    /// The inverse of Z-up point to Y-up.
    #[inline]
    pub fn point_ytozup(p: [f32; 3]) -> [f32; 3] {
        [p[0], -p[2], p[1]]
    }
}