pub mod defaults;

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, ErrorKind};
use std::path::Path;

use serde_derive::*;

use crate::addon::local::LocalAddon;
use crate::hard_error;
use defaults::*;

#[derive(Deserialize,Serialize,Default)]
pub struct Repo {
    pub conf: Conf,
    pub addons: Vec<LocalAddon>,
}

#[derive(Deserialize,Serialize)]
pub struct Conf {
    #[serde(default="default_url_txt")]
    pub url_txt: bool,
    #[serde(default="default_addon_mtime")]
    pub addon_mtime: bool,
    #[serde(default="default_soft_retries")]
    pub soft_retries: usize,
    #[serde(default="default_headers")]
    pub headers: Vec<(String,String)>,
    #[serde(default="default_domain")]
    pub domain: String,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            headers: default_headers(),
            domain: default_domain(),
            url_txt: default_url_txt(),
            addon_mtime: default_addon_mtime(),
            soft_retries: default_soft_retries(),
        }
    }
}

impl Repo {
    pub fn load(conf: impl AsRef<Path>) -> Self {
        let f = match std::fs::read_to_string(conf) {
            Ok(f) => f,
            Err(e) if e.kind() == ErrorKind::NotFound => return Self::default(),
            Err(e) => hard_error!("Failed to read config: {}",e),
        };
        match serde_json::from_str(&f) {
            Ok(c) => c,
            Err(e) => hard_error!("Failed to read config: {}",e),
        }
    }
    pub fn save(&self, conf: impl AsRef<Path>) {
        let f = file_write(conf).unwrap(); //TODO fix
        let f = BufWriter::new(f);
        serde_json::to_writer_pretty(f, self).unwrap();
    }
}

fn file_write(p: impl AsRef<Path>) -> std::io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(p)
}
fn file_read(p: impl AsRef<Path>) -> std::io::Result<File> {
    OpenOptions::new()
        .read(true)
        .open(p)
}
