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