use std::ffi::{OsStr, OsString};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use anyhow::bail;

use crate::{log_error, error};

#[must_use]
pub struct Finalize {
    pub finalize: Option<Box<dyn FnOnce() -> anyhow::Result<()>>>,
    pub cancel: Option<Box<dyn FnOnce()>>,
}

impl Finalize {
    pub fn for_part_path(path: PathBuf, part_path: PathBuf, finalized: bool) -> Self {
        let part_path2 = part_path.clone();
        if finalized {
            return Self::noop();
        }
        Self {
            finalize: Some(Box::new(move || {
                remove_if(&path)?;
                std::fs::rename(&part_path, &path)?;
                Ok(())
            })),
            cancel: Some(Box::new(move || {
                log_error!(remove_if(&part_path2));
            }))
        }
    }

    pub fn guard_file(path: PathBuf, did_it_exist: bool) -> Self {
        Self {
            finalize: None,
            cancel: (!did_it_exist).then(|| Box::new(move || {
                log_error!(remove_if(&path));
            }) as Box<dyn FnOnce()>),
        }
    }

    pub fn noop() -> Self {
        Self {
            finalize: None,
            cancel: None,
        }
    }

    pub fn is_noop(&self) -> bool {
        self.finalize.is_none() && self.cancel.is_none()
    }

    pub fn finalize(mut self) -> anyhow::Result<()> {
        self.cancel = None;
        if let Some(finalize) = self.finalize.take() {
            finalize()
        } else {
            Ok(())
        }
    }

    pub fn finalize_slice(s: &mut [Self]) -> anyhow::Result<()> {
        for v in s.iter_mut() {
            if let Some(finalize) = v.finalize.take() {
                finalize()?;
            }
        }
        for v in s {
            v.cancel = None;
        }
        Ok(())
    }

    pub fn finalize_drain(s: &mut Vec<Self>) -> anyhow::Result<()> {
        let result = Self::finalize_slice(&mut *s);
        s.clear();
        result
    }
}

impl Drop for Finalize {
    fn drop(&mut self) {
        if let Some(cancel) = self.cancel.take() {
            cancel()
        }
    }
}

impl std::ops::Add<Self> for Finalize {
    type Output = Self;

    fn add(mut self, mut rhs: Self) -> Self::Output {
        let l_fin = self.finalize.take();
        let r_fin = rhs.finalize.take();
        let l_can = self.cancel.take();
        let r_can = rhs.cancel.take();

        fn add_fns(a: Option<Box<dyn FnOnce()>>, b: Option<Box<dyn FnOnce()>>) -> Option<Box<dyn FnOnce()>> {
            if a.is_some() && b.is_some() {
                let (a,b) = (a.unwrap(),b.unwrap());
                Some(Box::new(move || {
                    a();
                    b();
                }))
            } else {
                a.or(b)
            }
        }
        
        fn add_fns_result(a: Option<Box<dyn FnOnce() -> anyhow::Result<()>>>, b: Option<Box<dyn FnOnce() -> anyhow::Result<()>>>) -> Option<Box<dyn FnOnce() -> anyhow::Result<()>>> {
            if a.is_some() && b.is_some() {
                let (a,b) = (a.unwrap(),b.unwrap());
                Some(Box::new(move || {
                    a()?;
                    b()
                }))
            } else {
                a.or(b)
            }
        }

        Self {
            finalize: add_fns_result(l_fin, r_fin),
            cancel: add_fns(l_can, r_can),
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

pub fn is_existing(p: impl AsRef<Path>) -> bool {
    if let Ok(meta) = p.as_ref().symlink_metadata() {
        true //meta.is_file() || meta.is_symlink() || meta.is_dir()
    } else {
        false
    }
}

pub fn is_file_or_symlink(p: impl AsRef<Path>) -> bool {
    if let Ok(meta) = p.as_ref().symlink_metadata() {
        meta.is_file() || meta.is_symlink()
    } else {
        false
    }
}

pub fn create_guarded_symlink(src: impl AsRef<Path>, dest: PathBuf) -> anyhow::Result<Finalize> {
    let src = src.as_ref();

    let old_meta = dest.symlink_metadata();
    let did_link_exist = old_meta.is_ok();

    if src != dest {
        let was_link_a_file = old_meta.as_ref().map_or(false, |meta| meta.is_file() || meta.is_symlink() );

        if was_link_a_file {
            std::fs::remove_file(&dest)?;
        }

        std::os::unix::fs::symlink(src, &dest)?;
    }

    Ok(Finalize {
        finalize: None,
        cancel: (!did_link_exist).then(|| Box::new(move || {
            log_error!(remove_if(&dest));
        }) as Box<dyn FnOnce()>)
    })
}

pub fn create_guarded_symlink_lazy(src: PathBuf, dest: PathBuf) -> anyhow::Result<Finalize> {
    if src != dest {
        match dest.symlink_metadata() {
            Ok(meta) => if meta.is_dir() {
                bail!("Directory");
            },
            Err(e) if e.kind() != ErrorKind::NotFound => bail!("{}",e),
            _ => {}
        }

        Ok(Finalize {
            finalize: Some(Box::new(move || {
                match std::fs::remove_file(&dest) {
                    Err(e) if e.kind() != ErrorKind::NotFound => bail!("{}",e),
                    _ => {}
                }

                std::os::unix::fs::symlink(src, &dest)?;

                Ok(())
            })),
            cancel: None,
        })
    } else {
        Ok(Finalize::noop())
    }
}

pub fn attached_to_path(path: impl Into<PathBuf>, add: impl AsRef<OsStr>) -> PathBuf {
    let mut path = path.into().into_os_string();
    path.push(add);
    path.into()
}
