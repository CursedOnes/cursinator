use std::ffi::{OsStr, OsString};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use crate::log_error;

#[must_use]
pub struct Finalize {
    path: PathBuf,
    part_path: PathBuf,
    finalized: bool,
}

impl Finalize {
    pub fn new(path: PathBuf, part_path: PathBuf, finalized: bool) -> Self {
        Self{path,part_path,finalized}
    }

    pub fn finalize(mut self) {
        if !self.finalized && self.part_path.exists() {
            log_error!(remove_if(&self.path));
            log_error!(std::fs::rename(&self.part_path, &self.path));
            self.finalized = true;
        }
    }
}

impl Drop for Finalize {
    fn drop(&mut self) {
        if !self.finalized {
            log_error!(remove_if(&self.part_path));
        }
    }
}


pub fn part_file_path(p: impl AsRef<OsStr>) -> PathBuf {
    let mut s: OsString = p.as_ref().to_owned();
    s.push(".part");
    PathBuf::from(s)
}


pub fn remove_if(path: impl AsRef<Path>) -> std::io::Result<bool> {
    match std::fs::remove_file(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e)
    }
}
