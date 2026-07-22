pub const MAGIC: &[u8; 5] = b"SNAPR";

pub const VERSION_1: u8 = 1;

pub const FLAG_NONE: u8 = 0;


//Total size of compressed object header
pub const HEADER_SIZE: usize = 16;

//directories and files
#[allow(unused)]
pub const SNAPR_DIR: &str = ".snapr";

pub const OBJECTS_DIR: &str = ".snapr/objects";

pub const SNAPSHOTS_FILE: &str = ".snapr/snapshots.json";

pub const WORKSPACE_INDEX_FILE: &str = ".snapr/workspace_index.json";

pub const CONFIG_FILE: &str = ".snapr/config.json";