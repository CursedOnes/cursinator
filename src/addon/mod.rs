pub mod files;
pub mod release_type;
pub mod dependency;
pub mod local;
pub mod download;
pub mod rtm;
pub mod validate;

use std::fmt::Display;

use serde::{Deserialize,Serialize,de::Error};

use crate::{error, hard_error};
use crate::util::version::{VersionMatcher, VersionPart};

#[derive(Deserialize,Serialize,Copy,Clone,PartialEq,Eq,Hash)]
#[serde(transparent)]
#[repr(transparent)]
pub struct AddonID(pub u64);

#[derive(Deserialize,Serialize,Clone)]
#[serde(transparent)]
#[repr(transparent)]
pub struct AddonSlug(pub String);

impl AddonSlug {
    pub fn from_string_ref(s: &String) -> &Self {
        unsafe {
            std::mem::transmute::<&String,&Self>(s)
        }
    }
}

#[derive(Deserialize,Serialize,Copy,Clone,PartialEq,Eq,Hash)]
#[serde(transparent)]
#[repr(transparent)]
pub struct FileID(pub u64);

#[derive(Clone)]
pub struct FileGameVersion {
    str: String,
    vpart: VersionPart,
}

#[derive(Clone)]
pub struct GameVersion {
    str: String,
    matcher: VersionMatcher,
}

impl FileGameVersion {
    pub fn from_string(mut str: String) -> Self {
        if str.ends_with("-Snapshot") || str.ends_with("-snapshot") {
            let mut v = str.into_bytes();
            v.truncate(v.len()-9);
            v.extend(b".99999999");
            str = String::from_utf8(v).unwrap();
        }

        let vpart = match VersionPart::parse_str(&str) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to parse file game version of addon: {e}");
                VersionPart::empty()
            },
        };

        // if vpart.points.is_empty() {
        //     dbg!(&str);
        //     error!("Configured game version must not be empty");
        // }

        Self {
            str,
            vpart
        }
    }
}

impl GameVersion {
    pub fn from_string(str: String) -> Self {
        let matcher = match VersionMatcher::parse(&str) {
            Ok(v) => v,
            Err(e) => {
                hard_error!("Filed to parse configured game version: {e}");
            },
        };

        if matcher.is_empty_recursive() {
            hard_error!("Configured game version must not be empty");
        }

        Self {
            str,
            matcher
        }
    }

    pub fn parse_and_match_str(&self, mut str: &str) -> bool {
        let mut str = str.to_owned();
        if str.ends_with("-Snapshot") || str.ends_with("-snapshot") {
            let mut v = str.into_bytes();
            v.truncate(v.len()-9);
            v.extend(b".99999999");
            str = String::from_utf8(v).unwrap();
        }

        let vpart = match VersionPart::parse_str(&str) {
            Ok(v) => v,
            Err(e) => {
                error!("Filed to parse configured game version: {e}");
                return false;
            },
        };

        // if vpart.points.is_empty() {
        //     dbg!(&str);
        //     error!("Configured game version must not be empty");
        // }
        
        self.matcher.matches_version(&vpart)
    }
}

impl<'de> serde::Deserialize<'de> for GameVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let str = String::deserialize(deserializer)?;
        
        Ok(GameVersion::from_string(str))
    }
}

impl serde::Serialize for GameVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        self.str.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for FileGameVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let str = String::deserialize(deserializer)?;
        
        Ok(Self::from_string(str))
    }
}

impl serde::Serialize for FileGameVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        self.str.serialize(serializer)
    }
}

impl PartialEq<FileGameVersion> for GameVersion {
    fn eq(&self, other: &FileGameVersion) -> bool {
        self.matcher.matches_version(&other.vpart)
    }
}

fn eq_str_concat<const N: usize>(a: &str, other: [&str;N]) -> bool {
    let mut a = a.as_bytes();
    for b in other {
        let b = b.as_bytes();
        if a.starts_with(b) {
            a = unsafe { a.get_unchecked(..b.len()) };
        } else {
            return false;
        }
    }
    a.is_empty()
}

impl GameVersion {
    pub fn matches<'a>(&self, mut gv: impl Iterator<Item=&'a FileGameVersion>) -> bool {
        gv.any(|v| self == v )
    }
    pub fn matches_idx<'a>(&self, gv: impl Iterator<Item=&'a FileGameVersion>) -> Option<usize> {
        gv.enumerate().find(|(_,v)| &self == v ).map(|(i,_)| i )
    }
}

impl PartialEq for AddonSlug {
    fn eq(&self, other: &Self) -> bool {
        self.0.trim() == other.0.trim()
    }
}

impl Display for AddonSlug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0,f)
    }
}
