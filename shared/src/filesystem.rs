use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
pub struct FileScan {
    pub directories: Vec<Directory>,
    pub files: Vec<File>,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct File {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Directory {
    pub name: String,
}

impl FileScan {
    pub fn new(dir: impl Into<std::path::PathBuf>) -> Option<Self> {
        let dir = dir.into();
        let mut s = FileScan::default();
        debug!("Scanning {:?}", dir);
        if !dir.exists() {
            return None;
        }
        for entry in dir.read_dir().ok()?.flatten() {
            let file_type = entry.file_type().ok()?;
            if file_type.is_dir() {
                let name = entry.file_name().into_string().unwrap();
                // println!("Adding '{name}' as a directory");
                s.directories.push(Directory { name })
            } else if file_type.is_file() {
                let name = entry.file_name().into_string().unwrap();
                // println!("Adding '{name}' as a file");
                s.files.push(File { name })
            }
        }

        // println!("Completed scan: {s:?}");
        Some(s)
    }

    pub fn has_dir(&self, target_dir: &str) -> bool {
        for dir in &self.directories {
            if dir.name == target_dir {
                return true;
            }
        }
        false
    }
    pub fn has_file(&self, target_dir: &str) -> bool {
        for file in &self.files {
            if file.name == target_dir {
                return true;
            }
        }
        false
    }
}
