use std::{
    fs::read_dir,
    path::{Path, PathBuf},
};

pub fn visit_dir_files(path: &Path, cb: &mut dyn FnMut(PathBuf)) {
    for e in read_dir(path).unwrap() {
        let path = e.unwrap().path();
        if path.is_file() {
            cb(path);
        }
    }
}
