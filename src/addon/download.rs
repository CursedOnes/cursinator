use std::fs::{File, OpenOptions};
use std::os::unix::prelude::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::io::Write;
use std::io::Read;
use std::str::FromStr;
use std::io::BufRead;
use std::time::Duration;

use crate::api::API;
use crate::conf::Conf;
use crate::*;
use crate::util::fs::Finalize;

use anyhow::anyhow;
use chrono::NaiveDateTime;
use filetime::{FileTime, set_file_times};
use sha1::{Digest, Sha1};
use util::fs::remove_if;

use super::files::AddonFile;

impl AddonFile {
    pub fn download(&self, conf: &Conf, api: &API) -> Result<DownloadFinalize,anyhow::Error> {
        let file_path = self.file_path();
        let url_txt_path = self.url_txt_path();
        let file_part_path = self.file_part_path();
        let url_txt_part_path = self.url_txt_part_path();

        let file_length = try_from!(self.file_length,anyhow!("file too big"));

        let mut soft_error: Option<anyhow::Error> = None;
        // retries on soft-errors like hash mismatch
        for retry_i in 0..conf.soft_retries.max(1) {
            if let Some(soft_error) = &soft_error {
                error!("Error: {soft_error}, retry download");
            }

            let mut buf = Vec::with_capacity(file_length);
            
            let resp = match api.http_get(&self.download_url.0) {
                Err(e @ ureq::Error::Status(429, _)) => {
                    error!("Retry download due to 429");
                    soft_error = Some(e.into());
                    std::thread::sleep(Duration::from_secs( 4u64.pow(retry_i.min(3)) ));
                    continue;
                }
                v => v,
            }?;

            resp.into_reader().read_to_end(&mut buf)?;

            // verify size and murmur hash of downloaded data
            soft_assert!(buf.len() == file_length, anyhow!("file_length mismatch"), soft_error);
            //soft_assert!(murmur32(&buf) == self.package_fingerprint, anyhow!("package_fingerprint mismatch"),soft_error);

            // write the downloaded data to mod.part
            let mut mod_file = file_write(&file_part_path)?;
            mod_file.write_all(&buf)?;

            // hash the downloaded data
            let sha = Sha1::from(&buf).digest();
            let sha_str = sha.to_string();

            if let Some(sha1_hash) = &self.sha1_hash {
                soft_assert!(sha_str == *sha1_hash, anyhow!("File Hash mismatch"), soft_error);
            }

            if conf.url_txt {
                // write .url.txt.part with file url and SHA1 hash
                let mut url_txt_file = file_write(&url_txt_part_path)?;
                writeln!(url_txt_file,"{}",self.download_url.0.trim())?;
                writeln!(url_txt_file,"{}",sha_str)?;
                url_txt_file.flush()?;
            }

            mod_file.flush()?;
            drop(mod_file);

            if conf.addon_mtime {
                // write addon publish time and current time to mtime and atime
                if let Some(addon_time) = log_error!(NaiveDateTime::parse_from_str(&self.file_date, "%Y-%m-%dT%H:%M:%S.%fZ")) {
                    let addon_time = FileTime::from_unix_time(addon_time.timestamp(),0);
                    let now = FileTime::now();
                    log_error!(set_file_times(&file_part_path, now, addon_time),   |e| "Failed to set file time: {}",e);
                    log_error!(set_file_times(&url_txt_part_path, now, addon_time),|e| "Failed to set file time: {}",e);
                }
            }

            return Ok(DownloadFinalize{
                file: Finalize::new(file_path,file_part_path,false),
                url_txt: Finalize::new(url_txt_path,url_txt_part_path,!conf.url_txt),
            });
        }
        Err(soft_error.unwrap())
    }

    pub fn remove(&self) -> anyhow::Result<bool> {
        remove_if(self.url_txt_path())?;
        Ok(remove_if(self.file_path())?)
    }

    pub fn file_path(&self) -> PathBuf {
        PathBuf::from(&self.file_name)
    }
    pub fn file_part_path(&self) -> PathBuf {
        PathBuf::from(format!("{}.part",self.file_name))
    }
    pub fn url_txt_path(&self) -> PathBuf {
        PathBuf::from(format!("{}.url.txt",self.file_name))
    }
    pub fn url_txt_part_path(&self) -> PathBuf {
        PathBuf::from(format!("{}.url.txt.part",self.file_name))
    }

    pub fn is_downloaded(&self) -> bool {
        self.file_path().is_file()
    }
}

#[must_use]
pub struct DownloadFinalize {
    file: Finalize,
    url_txt: Finalize,
}

impl DownloadFinalize {
    pub fn finalize(self) {
        self.file.finalize();
        self.url_txt.finalize();
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

#[macro_export]
macro_rules! soft_assert {
    ($oof:expr,$e:expr,$soft_error:ident) => {
        if !$oof {
            $soft_error = Some($e);
            continue
        }
    };
}
#[macro_export]
macro_rules! soft_error {
    ($oof:expr,$soft_error:ident) => {
        match $oof {
            Ok(f) => f,
            Err(e) => {
                $soft_error = Some(e.into());
                continue
            },
        }
    };
}
#[macro_export]
macro_rules! soft_result {
    ($oof:expr,$e:expr,$soft_error:ident) => {
        match $oof {
            Ok(f) => f,
            Err(_) => {
                $soft_error = Some($e);
                continue
            },
        }
    };
}
#[macro_export]
macro_rules! soft_option {
    ($oof:expr,$e:expr,$soft_error:ident) => {
        match $oof {
            Some(f) => f,
            None => {
                $soft_error = Some($e);
                continue
            },
        }
    };
}
#[macro_export]
macro_rules! soft_optres {
    ($oof:expr,$e:expr,$soft_error:ident) => {
        match $oof {
            Some(f) => match f {
                Ok(f) => f,
                Err(e) => {
                    $soft_error = Some(e.into());
                    continue
                },
            },
            None => {
                $soft_error = Some($e);
                continue
            },
        }
    };
}
#[macro_export]
macro_rules! try_from {
    ($v:expr,$e:expr) => {
        std::convert::TryFrom::try_from($v).map_err(|_| $e )?
    };
}

#[test]
fn parse_date() {
    NaiveDateTime::parse_from_str("2021-02-13T20:36:05.29Z","%Y-%m-%dT%H:%M:%S.%fZ").unwrap();
}

//TODO test murmur2
