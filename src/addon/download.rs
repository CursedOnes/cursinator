use std::fs::{File, OpenOptions};
use std::os::unix::prelude::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::io::Write;
use std::io::Read;
use std::str::FromStr;
use std::io::BufRead;

use crate::api::API;
use crate::conf::Conf;
use crate::*;

use anyhow::anyhow;
use chrono::NaiveDateTime;
use filetime::{FileTime, set_file_times};
use sha1::{Digest, Sha1};
use util::remove_if;

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
        for _ in 0..conf.soft_retries.max(1) {
            let mut buf = Vec::with_capacity(file_length);
            
            let resp = api.http_get(&self.download_url.0)?;
            resp.into_reader().read_to_end(&mut buf)?;

            // verify size and murmur hash of downloaded data
            soft_assert!(buf.len() == file_length, anyhow!("file_length mismatch"),soft_error);
            soft_assert!(murmur32(&buf) == self.package_fingerprint, anyhow!("package_fingerprint mismatch"),soft_error);

            // write the downloaded data to mod.part
            let mut mod_file = file_write(&file_part_path)?;
            mod_file.write_all(&buf)?;

            // hash the downloaded data
            let sha = Sha1::from(&buf).digest();

            if conf.url_txt {
                // write .url.txt.part with file url and SHA1 hash
                let mut url_txt_file = file_write(&url_txt_part_path)?;
                write!(url_txt_file,"{}\n",self.download_url.0.trim())?;
                write!(url_txt_file,"{}\n",sha.to_string())?;
                url_txt_file.flush()?;
            }

            mod_file.flush()?;
            drop(mod_file);

            // verify size of written file
            //let file_part_len: usize = try_from!(file_part_path.metadata()?.len(),anyhow!("file too big"));
            //soft_assert!(file_part_len == buf.len(), anyhow!("local_file_length mismatch"),soft_error);

            // read back file into buffer
            let mut mod_file = file_read(&file_part_path)?;
            soft_error!(mod_file.read_exact(&mut buf),soft_error);
            drop(mod_file);

            // hash-verify read-back data
            soft_assert!(murmur32(&buf) == self.package_fingerprint, anyhow!("package_fingerprint mismatch"),soft_error);
            soft_assert!(Sha1::from(&buf).digest() == sha, anyhow!("sha mismatch"),soft_error);

            if conf.url_txt {
                // read back written .url.txt file and verify url and hash
                let url_text = {
                    let mut url_txt_file = file_read(&url_txt_part_path)?;
                    let mut v = Vec::with_capacity(4096);
                    url_txt_file.read_to_end(&mut v)?;
                    v
                };
                let mut lines = url_text.lines();
                let line_url = soft_optres!(lines.next(), anyhow!("akw"),soft_error);
                let line_hash = soft_optres!(lines.next(), anyhow!("akw"),soft_error);

                soft_assert!(line_url.trim() == self.download_url.0.trim(), anyhow!("url_text_url mismatch"),soft_error);
                let sha2 = soft_result!(Digest::from_str(line_hash.trim()), anyhow!("url_text_hash mismatch"),soft_error);
                soft_assert!(sha2 == sha, anyhow!("url_text_hash mismatch"),soft_error);
            }

            if conf.addon_mtime {
                // write addon publish time and current time to mtime and atime
                if let Some(addon_time) = log_error!(NaiveDateTime::parse_from_str(&self.file_date, "%Y-%m-%dT%H:%M:%S.%fZ")) {
                    let addon_time = FileTime::from_unix_time(addon_time.timestamp(),0);
                    let now = FileTime::now();
                    log_error!(set_file_times(&file_part_path, now, addon_time),   |e| "Failed to set file time: {}",e);
                    log_error!(set_file_times(&url_txt_part_path, now, addon_time),|e| "Failed to set file time: {}",e);
                }
            }

            return Ok(DownloadFinalize{file_path,file_part_path,url_txt_path,url_txt_part_path});
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
}

#[must_use]
pub struct DownloadFinalize {
    file_path: PathBuf,
    file_part_path: PathBuf,
    url_txt_path: PathBuf,
    url_txt_part_path: PathBuf,
}

impl Drop for DownloadFinalize {
    fn drop(&mut self) {
        // move .part to final files
        log_error!(remove_if(&self.file_path));
        log_error!(std::fs::rename(&self.file_part_path, &self.file_path));
        log_error!(remove_if(&self.url_txt_path));
        log_error!(std::fs::rename(&self.url_txt_part_path, &self.url_txt_path));
    }
}

fn file_write(p: impl AsRef<Path>) -> std::io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .custom_flags(libc::O_DIRECT)
        .open(p)
}
fn file_read(p: impl AsRef<Path>) -> std::io::Result<File> {
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECT)
        .open(p)
}

fn murmur32(buf: &[u8]) -> u32 {
    murmur32_seed(buf,0)
}

fn murmur32_seed(mut buf: &[u8], seed: u32) -> u32 {
    // stolen from smhasher: https://github.com/aappleby/smhasher/blob/61a0530f28277f2e850bfc39600ce61d02b518de/src/MurmurHash2.cpp#L37

    // 'm' and 'r' are mixing constants generated offline.
    // They're not really 'magic', they just happen to work well.

    let m: u32 = 0x5bd1e995;
    let r: u32= 24;

    // Initialize the hash to a 'random' value

    let mut h: u32 = seed ^ (buf.len() as u32);

    // Mix 4 bytes at a time into the hash

    while buf.len() >= 4 {
        let mut k = u32::from_le_bytes([buf[0],buf[1],buf[2],buf[3]]);

        k *= m;
        k ^= k >> r;
        k *= m;

        h *= m;
        h ^= k;

        buf = &buf[4..];
    }

    // Handle the last few bytes of the input array

    match buf.len() {
        3 => h ^= (buf[2] as u32) << 16,
        2 => h ^= (buf[1] as u32) << 8,
        1 => h ^= buf[0] as u32,
        _ => {},
    }

    h *= m;

    // Do a few final mixes of the hash to ensure the last few
    // bytes are well-incorporated.

    h ^= h >> 13;
    h *= m;
    h ^= h >> 15;

    h
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
fn dastgerherg() {
    NaiveDateTime::parse_from_str("2021-02-13T20:36:05.29Z","%Y-%m-%dT%H:%M:%S.%fZ").unwrap();
}

//TODO test murmur2