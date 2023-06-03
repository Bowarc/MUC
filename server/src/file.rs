// Copied straight from my game Vupa

use std::io::Read as _;

// #[derive(rust_embed::RustEmbed)]
// #[folder = "$CARGO_MANIFEST_DIR\\resources\\internal\\"]
// struct Resolver;

#[derive(Debug)]
pub enum FileSystem {
    // Internal, // there is no internal file system here, every file is mutable
    External,
}

#[derive(Debug)]
pub struct Path {
    fs: FileSystem,
    p: String,
}

#[derive(Debug)]
pub struct ConsPath {
    fs: FileSystem,
    p: &'static str,
}

impl Path {
    pub fn new(fs: FileSystem, p: &'static str) -> Self {
        Self {
            fs,
            p: p.to_string(),
        }
    }

    pub fn build(&self) -> String {
        format!("{}\\files\\{}", env!("CARGO_MANIFEST_DIR"), self.p)
    }
}

impl ConsPath {
    pub const fn new(fs: FileSystem, p: &'static str) -> Self {
        Self { fs, p }
    }
}

impl std::convert::From<ConsPath> for Path {
    fn from(o: ConsPath) -> Path {
        Path {
            fs: o.fs,
            p: o.p.to_string(),
        }
    }
}

impl std::ops::Add<&str> for Path {
    type Output = Self;

    fn add(self, other: &str) -> Path {
        Self {
            fs: self.fs,
            p: self.p + other,
        }
    }
}

pub fn try_load_bytes(path: Path) -> Result<std::borrow::Cow<'static, [u8]>, std::io::Error> {
    let stopwatch = std::time::SystemTime::now();

    let start_info_message = format!("Loading {:?} {}", path.fs, path.p);
    match path.fs {
        // FileSystem::Internal => match Resolver::get(&path.p) {
        //     Some(file) => {
        //         debug!(
        //             "{} . . success in {}",
        //             start_info_message,
        //             display_duration(stopwatch.elapsed().unwrap(), "")
        //         );
        //         Ok(file.data)
        //     }
        //     None => {
        //         let error = std::io::Error::new(
        //             std::io::ErrorKind::Other,
        //             format!("Could not open path: {:?}, {:?}", path.fs, path.p),
        //         );
        //         error!("{} . . error: {error}", start_info_message);
        //         Err(error)
        //     }
        // },
        FileSystem::External => {
            match std::fs::File::open(path.build()) {
                Ok(mut file) => {
                    let mut bytes: Vec<u8> = Vec::new();
                    let _ = file.read_to_end(&mut bytes);

                    debug!(
                        "{} . . success in {}",
                        start_info_message,
                        display_duration(stopwatch.elapsed().unwrap(), "")
                    );
                    Ok(bytes.into())
                }
                Err(e) => {
                    // format!("Could not open path: {:?}, {}", path.fs, path.p);
                    error!("{} . . error: {e}", start_info_message);
                    Err(e)
                }
            }
        }
    }
}
pub fn load_bytes(path: Path) -> std::borrow::Cow<'static, [u8]> {
    try_load_bytes(path).unwrap()
}

pub fn write_bytes(path: Path, content: impl AsRef<[u8]>) -> Result<(), ()> {
    let p = path.build();

    std::fs::write(p, content).map_err(|_| ())?;

    Ok(())
}

// This is supposed to be in a time module (at least in my game, but idc for now)
pub fn display_duration(d: std::time::Duration, separator: &str) -> String {
    let mut value: f64 = d.as_nanos() as f64;

    let units: Vec<&str> = vec!["ns", "µs", "ms", "s"];
    let mut name_index = 0;

    while value > 1_000. {
        if name_index < units.len() - 1 {
            value /= 1_000.;
            name_index += 1
        } else {
            break;
        }
    }

    format!("{:.2}{}{}", value, separator, units[name_index])
}
