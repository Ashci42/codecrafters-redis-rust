use std::path::Path;

pub struct Config<'a> {
    pub addr: &'a str,
    pub rdb_dir: Option<&'a Path>,
    pub rdb_file_name: Option<&'a str>,
}

impl<'a> Config<'a> {
    pub fn new(addr: &'a str, rdb_dir: Option<&'a Path>, rdb_file_name: Option<&'a str>) -> Self {
        Self {
            addr,
            rdb_dir,
            rdb_file_name,
        }
    }
}
