pub mod files;
pub mod release_type;
pub mod dependency;
pub mod local;
pub mod download;
pub mod rtm;

use std::fmt::Display;

use serde::{Deserialize,Serialize};

#[derive(Deserialize,Serialize,Copy,Clone,PartialEq,Eq,Hash)]
#[serde(transparent)]
#[repr(transparent)]
pub struct AddonID(pub u64);

#[derive(Deserialize,Serialize,Clone)]
#[serde(transparent)]
#[repr(transparent)]
pub struct AddonSlug(pub String);

#[derive(Deserialize,Serialize,Copy,Clone,PartialEq,Eq,Hash)]
#[serde(transparent)]
#[repr(transparent)]
pub struct FileID(pub u64);

#[derive(Deserialize,Serialize,Clone)]
#[serde(transparent)]
#[repr(transparent)]
pub struct FileGameVersion(pub String);

#[derive(Deserialize,Serialize,Clone)]
#[serde(transparent)]
#[repr(transparent)]
pub struct GameVersion(pub String);

impl PartialEq<GameVersion> for FileGameVersion {
    fn eq(&self, other: &GameVersion) -> bool {
        other.eq(self)
    }
}
impl PartialEq<FileGameVersion> for GameVersion {
    fn eq(&self, other: &FileGameVersion) -> bool {
        let s = self.0.trim();
        let other = other.0.trim();
        let s: Vec<_> = s.splitn(2,'x').collect();
        if s.len() > 2 {
            panic!("Invalid game version pattern");
        } else if s.len() == 2 {
            other.starts_with(s[0]) && other.ends_with(s[1])
        } else if s.len() == 1 {
            other == s[0]
        } else {
            panic!("Invalid game version pattern");
        }
    }
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
