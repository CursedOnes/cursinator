use std::fs::{File, OpenOptions};
use std::os::unix::prelude::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::io::Write;
use std::io::Read;
use std::str::FromStr;
use std::io::BufRead;
use std::time::Duration;

use crate::api::{API, parse_retry_duration};
use crate::conf::Conf;
use crate::*;
use crate::util::fs::{Finalize, is_existing, is_file_or_symlink, create_guarded_symlink, attached_to_path, create_guarded_symlink_lazy};

use anyhow::{anyhow, bail};
use chrono::DateTime;
use filetime::{FileTime, set_file_times};
use sha1::{Sha1, Digest};
use util::fs::remove_if;

use super::AddonID;
use super::files::AddonFile;

impl AddonFile {
    pub fn download(&self, paths: &FilePaths, conf: &Conf, api: &mut API, cache_only: bool) -> Result<Finalize,anyhow::Error> {
        conf.ensure_cache_dir()?;

        let file_length = try_from!(self.file_length,anyhow!("file too big"));

        let mut validated = None;
        
        match self.is_downloaded_valid(paths) {
            Ok(v) => validated = v,
            Err(e) if e.downcast_ref::<std::io::Error>()
                .map_or(false, |e| e.kind() == std::io::ErrorKind::NotFound ) => {},
            Err(e) => warn!("{}",e),
        }

        let download_to = paths.cache_path.as_ref().unwrap_or(&paths.part_path);
        
        assert_eq!(paths.cache_path.is_some(), conf.symlink_cache_path.is_some());

        let mut soft_error: Option<anyhow::Error> = None;
        // retries on soft-errors like hash mismatch
        for retry_i in 0..conf.soft_retries.max(1) {
            if let Some(soft_error) = &soft_error {
                error!("Error: {soft_error}, retry download");
            }

            if validated.is_none() {
                let mut buf = Vec::with_capacity(file_length);

                let download_url = self.download_url.as_ref()
                    .ok_or_else(|| anyhow::anyhow!("No download link") )?;
                
                let resp = match api.http_get(&download_url.0) {
                    Err(e) => {
                        if let ureq::Error::Status(429, response) = &e {
                            let wait_duration = parse_retry_duration(
                                response.header("Retry-After"),
                                4u64.pow(retry_i.min(3)),
                            );
                            error!("Too many requests, retry in {wait_duration} seconds");
                            soft_error = Some(e.into());
                            std::thread::sleep(Duration::from_secs( 4u64.pow(retry_i.min(3)) ));
                            continue;
                        } else {
                            Err(e)
                        }
                    }
                    v => v,
                }?;

                resp.into_reader().read_to_end(&mut buf)?;

                soft_assert!(buf.len() == file_length, anyhow!("file_length mismatch"), soft_error);

                // hash the downloaded data
                let sha = {
                    let mut hasher = Sha1::new();
                    hasher.update(&buf);
                    hasher.finalize()
                };
                let sha_str = hex::encode(&*sha);

                if let Some(sha1_hash) = &self.sha1_hash {
                    soft_assert!(sha_str == *sha1_hash, anyhow!("File Hash mismatch"), soft_error);
                }

                std::fs::write(download_to, &buf)?;

                validated = Some(sha_str);
            }
            
            if cache_only {
                return Ok(Finalize::noop());
            }

            let mut finalizer = if let Some(cache_path) = &paths.cache_path {
                create_guarded_symlink_lazy(cache_path.clone(), paths.path.clone())?
            } else {
                Finalize::for_part_path(paths.path.clone(), download_to.clone(), false)
            };

            if conf.url_txt {
                finalizer = finalizer + self.write_url_txt(paths, conf, api, validated.as_ref().unwrap())?;
            }

            if conf.addon_mtime {
                // write addon publish time and current time to mtime and atime
                if let Some(addon_time) = log_error!(parse_date(&self.file_date)) {
                    let addon_time = FileTime::from_unix_time(addon_time.timestamp(),0);
                    let now = FileTime::now();
                    if let Some(cache_path) = &paths.cache_path {
                        log_error!(set_file_times(cache_path, now, addon_time),   |e| "Failed to set file time for cache_path: {}",e);
                    } else {
                        log_error!(set_file_times(&paths.part_path, now, addon_time),   |e| "Failed to set file time for path: {}",e);
                    }
                    if paths.url_txt_path.is_file() {
                        log_error!(set_file_times(&paths.url_txt_path, now, addon_time),|e| "Failed to set file time url_txt_path: {}",e);
                    }
                }
            }

            return Ok(finalizer);
        }
        Err(soft_error.unwrap())
    }

    pub fn write_url_txt(&self, paths: &FilePaths, conf: &Conf, api: &mut API, sha: &str) -> Result<Finalize,anyhow::Error> {
        // write .url.txt.part with file url and SHA1 hash
        let mut url_txt = vec![];

        let download_url = self.download_url.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No download link") )?;

        writeln!(&mut url_txt,"{}",download_url.0.trim())?;
        writeln!(&mut url_txt,"{}",sha)?;

        let did_it_exist = is_existing(&paths.url_txt_path);

        std::fs::write(&paths.url_txt_path, url_txt)?;

        Ok(Finalize::guard_file(paths.url_txt_path.clone(), did_it_exist))
    }

    pub fn file_paths_part_new(&self, disabled: bool) -> FilePathsPart {
        if disabled {
            FilePathsPart {
                path: attached_to_path(&self.file_name, ".disabled"),
                part_path: attached_to_path(&self.file_name, ".disabled.part"),
                url_txt_path: attached_to_path(&self.file_name, ".disabled.url.txt"),
                disabled,
            }
        } else {
            FilePathsPart {
                path: (&self.file_name).into(),
                part_path: attached_to_path(&self.file_name, ".part"),
                url_txt_path: attached_to_path(&self.file_name, ".url.txt"),
                disabled,
            }
        }
    }

    pub fn file_paths_part_current(&self, allow_fixups: bool) -> FilePathsPart {
        let disabled_path = attached_to_path(&self.file_name, ".disabled");
        let path = (&self.file_name).into();
        let disabled_url_txt = attached_to_path(&self.file_name, ".disabled.url.txt");
        let url_txt = attached_to_path(&self.file_name, ".url.txt");
        let part_path = attached_to_path(&self.file_name, ".part");

        let disabled = !is_existing(&path) && is_file_or_symlink(&disabled_path);
        
        // sync url.txt disabled status to file
        if disabled {
            if !is_existing(&disabled_url_txt) && is_file_or_symlink(&url_txt) && allow_fixups {
                log_error!(std::fs::rename(&url_txt, &disabled_url_txt));
            }
        } else {
            if !is_existing(&url_txt) && is_file_or_symlink(&disabled_url_txt) && allow_fixups {
                log_error!(std::fs::rename(&disabled_url_txt, &url_txt));
            }
        }

        if disabled {
            FilePathsPart {
                path: disabled_path,
                part_path,
                url_txt_path: disabled_url_txt,
                disabled,
            }
        } else {
            FilePathsPart {
                path,
                part_path,
                url_txt_path: url_txt,
                disabled,
            }
        }
    }

    pub fn file_paths_new(&self, addon_id: AddonID, disabled: bool, conf: &Conf) -> FilePaths {
        let paths = self.file_paths_part_new(disabled);

        let cache_path = conf.symlink_cache_path.as_ref().map(|cache_dir| {
            cache_dir.join(format!("cf_{}_{}_{}",addon_id.0,self.id.0,self.file_name))
        });

        FilePaths {
            path: paths.path,
            part_path: paths.part_path,
            cache_path,
            url_txt_path: paths.url_txt_path,
            disabled: paths.disabled,
        }
    }

    pub fn file_paths_current(&self, addon_id: AddonID, allow_fixups: bool, conf: &Conf) -> FilePaths {
        let paths = self.file_paths_part_current(allow_fixups);

        let cache_path = conf.symlink_cache_path.as_ref().map(|cache_dir| {
            cache_dir.join(format!("cf_{}_{}_{}",addon_id.0,self.id.0,self.file_name))
        });

        FilePaths {
            path: paths.path,
            part_path: paths.part_path,
            cache_path,
            url_txt_path: paths.url_txt_path,
            disabled: paths.disabled,
        }
    }
}

pub struct FilePaths {
    pub path: PathBuf,
    pub part_path: PathBuf,
    pub cache_path: Option<PathBuf>,
    pub url_txt_path: PathBuf,
    pub disabled: bool,
}

pub struct FilePathsPart {
    pub path: PathBuf,
    pub part_path: PathBuf,
    pub url_txt_path: PathBuf,
    pub disabled: bool,
}

impl FilePathsPart {
    pub fn remove(&self) -> anyhow::Result<bool> {
        remove_if(&self.url_txt_path)?;
        Ok(remove_if(&self.path)?)
    }

    pub fn is_downloaded(&self) -> bool {
        self.path.is_file()
    }
}

impl FilePaths {
    pub fn remove(&self) -> anyhow::Result<()> {
        remove_if(&self.url_txt_path)?;
        remove_if(&self.path)?;
        Ok(())
    }

    pub fn remove_if_not_new(&self, new: &Self) -> anyhow::Result<()> {
        if new.url_txt_path != self.url_txt_path {
            remove_if(&self.url_txt_path)?;
        }
        if new.path != self.path {
            remove_if(&self.path)?;
        }
        Ok(())
    }

    pub fn is_downloaded(&self) -> bool {
        self.path.is_file()
    }
}

pub(super) fn file_write(p: impl AsRef<Path>) -> std::io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(p)
}
pub(super) fn file_read(p: impl AsRef<Path>) -> std::io::Result<File> {
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

fn parse_date(s: &str) -> Result<DateTime<chrono::FixedOffset>,chrono::ParseError> {
    if let Ok(v) = DateTime::parse_from_rfc3339(s) {
        return Ok(v);
    }
    //TODO properly handle cases like "0001-01-01T00:00:00"
    if let Ok(v) = DateTime::parse_from_rfc3339(&format!("{s}Z")) {
        return Ok(v);
    }
    chrono::DateTime::parse_from_rfc3339(s)
}

#[test]
fn parse_date_test() {
    parse_date("2021-02-13T20:36:05Z").unwrap();
    parse_date("2021-02-13T20:36:05Z").unwrap();
}
