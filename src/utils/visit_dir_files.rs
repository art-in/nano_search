use std::{
    fs::read_dir,
    path::{Path, PathBuf},
};

pub fn visit_dir_files(path: &Path, cb: &mut dyn FnMut(PathBuf)) {
    for e in read_dir(path).expect("dir should be readable") {
        let path = e.expect("all dir entries should be accessible").path();
        if path.is_file() {
            cb(path);
        }
    }
}
