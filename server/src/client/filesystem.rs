const USER_FILES_ROOT: crate::file::ConsPath = crate::file::ConsPath::new("user_files/");

pub struct FileSystem {
    current_directory: UserPath,
    pub current_dir_scan: shared::filesystem::FileScan,
    user_id: String,
}

#[derive(Clone, Debug)]
pub struct UserPath {
    id: String,
    bits: Vec<UserPathBit>,
}

#[derive(Debug, Clone)]
pub enum UserPathBit {
    Directory(String),
    File(String),
}

impl FileSystem {
    pub fn new(user_id: uuid::Uuid) -> FileSystem {
        // let root_path: crate::file::Path = ;
        let str_id = user_id.hyphenated().to_string();

        let root_path = FileSystem::root_path(str_id.clone());

        FileSystem {
            current_directory: root_path.clone(),
            current_dir_scan: shared::filesystem::FileScan::new(root_path.build()).unwrap(),
            user_id: str_id,
        }
    }
    pub fn root_path(user_id: String) -> UserPath {
        UserPath::new_root(user_id)
    }
    pub fn is_root(&self) -> bool {
        self.current_directory.is_root()
    }

    /// returns a list of every element in current directory
    fn scan(&self) -> shared::filesystem::FileScan {
        shared::filesystem::FileScan::new(self.current_directory.build()).unwrap()
    }
    /// changes directory
    pub fn cd(&mut self, new_dir: String) {
        let actions = new_dir.split('/');

        #[allow(clippy::collapsible_else_if)]
        for action in actions {
            if action != ".." && self.current_dir_scan.has_dir(action) {
                self.current_directory
                    .push(UserPathBit::Directory(action.to_string()));
                self.current_dir_scan = self.scan()
            } else {
                if !self.is_root() {
                    let parent = self.current_directory.parent().unwrap();
                    self.current_directory = parent;
                    self.current_dir_scan = self.scan()
                } else {
                    println!("Can't do up further, you're already at root")
                }
            }
        }

        println!("Moved into {new_dir}")
    }
    /// retruns the content of a file in the directory
    pub fn read_file(&self, file_name: &str) {
        if self.current_dir_scan.has_file(file_name) {
            // read the file and returns the content
        }
    }
    /// write something to a file in the directory
    pub fn write_file(&self) {}

    // create a file in the current directory
    pub fn create_file(&self, file_name: &str) {}
}

impl UserPath {
    pub fn new_root(user_id: String) -> UserPath {
        UserPath {
            id: user_id,
            bits: Vec::new(),
        }
    }

    pub fn new(user_id: String, mut p: String) -> UserPath {
        let mut o = UserPath::new_root(user_id);

        let root_dir = crate::file::Path::from(USER_FILES_ROOT).build();
        p = p.replace(&root_dir, "");

        let clean_p = UserPath::clean_it(p);

        for bit in clean_p.split('/') {
            if bit.is_empty() {
                continue;
            }

            let mut builded = o.build();

            let buildedpb = std::path::PathBuf::from(builded.clone());

            if !buildedpb.exists() {
                println!("{buildedpb:?} doesn't exist");
                break;
            } else if buildedpb.is_file() {
                // no need to go further, a file can't be a parent
                break;
            }

            builded.push_str(bit);

            if std::path::PathBuf::from(builded.clone()).is_file() {
                o.push(UserPathBit::File(bit.to_string()))
            } else if std::path::PathBuf::from(builded).is_dir() {
                o.push(UserPathBit::Directory(bit.to_string()))
            } else {
                // I do not support links
                break;
            }

            println!("bit: {bit}")
        }

        o
    }
    fn clean_it(mut p: String) -> String {
        // This is to remove a 'bug?' with pathbuf on windows
        p = p.replace("\\\\?\\", "");
        p = p.replace('\\', "/").replace("//", "/");
        p
    }

    pub fn build(&self) -> String {
        let mut o = UserPath::clean_it(crate::file::Path::from(USER_FILES_ROOT).build());

        o.push_str(&format!("{}/", self.id));
        for bit in &self.bits {
            debug!("{o}");
            match bit {
                UserPathBit::Directory(name) => {
                    o.push_str(name);
                    o.push('/')
                }
                UserPathBit::File(name) => {
                    o.push_str(name);
                }
            }
        }
        o
    }
    pub fn push(&mut self, p: UserPathBit) {
        self.bits.push(p);
    }
    pub fn pop(&mut self) -> Option<UserPathBit> {
        self.bits.pop()
    }

    pub fn exists(&self) -> bool {
        let pbuff = std::path::PathBuf::from(self.build());

        pbuff.exists()
    }

    pub fn is_root(&self) -> bool {
        self.bits.is_empty()
    }

    pub fn parent(&self) -> Option<UserPath> {
        if self.is_root() {
            return None;
        }

        let mut parent = self.clone();
        parent.pop();
        Some(parent)
    }
}

#[test]
fn dir() {
    shared::logger::init(log::LevelFilter::Off, None);

    let mut files = FileSystem::new(uuid::uuid!("baa2b3e9-bbef-4cc6-b82b-a42654edf87d"));

    files.cd("test_dir/..".to_string());

    // files.cd("..".to_string());

    println!("{:?}", files.scan());

    println!("Done")
}

#[test]
fn path() {
    let p = std::path::PathBuf::from(crate::file::Path::from(USER_FILES_ROOT).build())
        .display()
        .to_string();

    println!("{p}");
    let mut user_path = UserPath::new(
        "baa2b3e9-bbef-4cc6-b82b-a42654edf87d".to_string(),
        "".to_string(),
    );
    println!("total bits: {:?}", user_path.bits);

    println!("{}", user_path.build());

    println!("Valid: {}", user_path.exists())
}
