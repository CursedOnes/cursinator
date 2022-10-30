pub mod defaults;

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, ErrorKind};
use std::path::Path;

use serde_derive::*;

use crate::addon::GameVersion;
use crate::addon::local::LocalAddons;
use crate::util::fs::{part_file_path, remove_if};
use defaults::*;

#[derive(Deserialize,Serialize)]
pub struct Repo {
    pub conf: Conf,
    pub addons: LocalAddons,
}

#[derive(Deserialize,Serialize)]
pub struct Conf {
    pub game_version: GameVersion,
    #[serde(default="default_url_txt")]
    pub url_txt: bool,
    #[serde(default="default_addon_mtime")]
    pub addon_mtime: bool,
    #[serde(default="default_soft_retries")]
    pub soft_retries: u32,
    #[serde(default="default_api_headers")]
    pub api_headers: Vec<(String,String)>,
    #[serde(default="default_api_domain")]
    pub api_domain: String,
}

impl Repo {
    pub fn load(conf: impl AsRef<Path>) -> anyhow::Result<Option<Self>> {
        let f = match std::fs::read_to_string(conf) {
            Ok(f) => f,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        match serde_json::from_str(&f) {
            Ok(c) => Ok(Some(c)),
            Err(e) => Err(e.into()),
        }
    }
    pub fn save(&self, conf: impl AsRef<Path>) -> anyhow::Result<()> {
        let conf_part = part_file_path(conf.as_ref());
        let f = file_write(&conf_part)?; //TODO fix, .part
        let f = BufWriter::new(f);
        serde_json::to_writer_pretty(f, self)?;
        remove_if(&conf)?;
        std::fs::rename(conf_part, conf)?;
        Ok(())
    }
    pub fn save_new(&self, conf: impl AsRef<Path>) -> anyhow::Result<()> {
        let f = file_write_new(conf)?; //TODO fix
        let f = BufWriter::new(f);
        serde_json::to_writer_pretty(f, self)?;
        Ok(())
    }
    pub fn sort_deps(&mut self) {
        for a in self.addons.values_mut() {
            if let Some(file) = &mut a.installed {
                file.sort_deps();
            }
        }
    }
}

fn file_write(p: impl AsRef<Path>) -> std::io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(p)
}
fn file_write_new(p: impl AsRef<Path>) -> std::io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(p)
}
