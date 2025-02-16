use std::{
    fs::read_dir,
    path::{Path, PathBuf},
};

pub fn visit_dir_files(path: &Path, cb: &mut dyn FnMut(PathBuf)) {
    let dir = read_dir(path).unwrap_or_else(|_| {
        panic!(
            "dir {0} should exist",
            path.to_str().expect("path should not be empty")
        )
    });
    for e in dir {
        let path = e.expect("all dir entries should be accessible").path();
        if path.is_file() {
            cb(path);
        }
    }
}
