use std::fmt::Display;
use std::ops::BitOr;
use serde_derive::*;

use crate::conf::Conf;

use super::GameVersion;
use super::files::AddonFile;
use super::release_type::ReleaseType;

/// Examples
/// release | beta | alpha | result
/// ------- | ---- | ----- | -
/// false | false | true | will pick the latest version
/// true | true | true | will pick the latest release, or the latest beta if there is no release, or the latest if there is no beta or release
/// true | false | true | will pick the latest release, or the latest (alpha or beta) if there is no release
/// false | true | true | will pick the latest beta, or the latest (alpha) if there is no beta
/// 
#[derive(Deserialize,Serialize,Clone,Copy,PartialEq)]
pub struct ReleaseTypeMode {
    pub release: bool, //TODO warn if (false,false,false)
    pub beta: bool,
    pub alpha: bool,
}

impl ReleaseTypeMode {
    pub fn new(mut release: bool, mut beta: bool, mut alpha: bool) -> Self {
        if !(release|beta|alpha) {
            release=true; beta=true; alpha=true;
        }
        Self{release,beta,alpha}
    }
    pub fn new2(release: bool, beta: bool, alpha: bool) -> Option<Self> {
        if !(release|beta|alpha) {
            return None;
        }
        Some(Self{release,beta,alpha})
    }
    pub fn legal(&self, r: ReleaseType) -> bool {
        match r {
            ReleaseType::Release => self.alpha|self.beta|self.release,
            ReleaseType::Beta    => self.alpha|self.beta,
            ReleaseType::Alpha   => self.alpha,
        }
    }
    pub fn pick_version<'a>(&self, v: &'a [AddonFile], conf: &Conf, blacklist: Option<&str>, positive_negative_in_filename: bool) -> Option<&'a AddonFile> {
        fn find_legal<'a>(v: &'a [AddonFile], g: ReleaseType, conf: &Conf, blacklist: Option<&str>, positive_negative_in_filename: bool) -> Option<&'a AddonFile> {
            v.iter()
                .rev()
                .filter(|file| conf.filter_addon_file(file, blacklist, positive_negative_in_filename))
                .find(|v| v.release_type >= g )
        }

        let mut r = None;

        if r.is_none() && self.release {
            r = find_legal(v, ReleaseType::Release, conf, blacklist, positive_negative_in_filename);
        }
        if r.is_none() && self.beta {
            r = find_legal(v, ReleaseType::Beta   , conf, blacklist, positive_negative_in_filename);
        }
        if r.is_none() && self.alpha {
            r = find_legal(v, ReleaseType::Alpha  , conf, blacklist, positive_negative_in_filename);
        }

        if r.is_none() {
            r = v.iter().last();
        }

        r
    }
    pub fn pick_level(&self, v: impl Iterator<Item=ReleaseType>+DoubleEndedIterator) -> ReleaseType {
        let (
            mut alpha_found,
            mut beta_found,
            mut release_found
        ) = Default::default();

        for v in v.rev() {
            match v {
                ReleaseType::Release => release_found = true,
                ReleaseType::Beta    => beta_found    = true,
                ReleaseType::Alpha   => alpha_found   = true,
            }
        }

        if self.release && release_found                        {return ReleaseType::Release;}
        if self.beta    && release_found|beta_found             {return ReleaseType::Beta;}
        if self.alpha   && release_found|beta_found|alpha_found {return ReleaseType::Alpha;}
        
        ReleaseType::Alpha
    }
}

impl Display for ReleaseTypeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}{}{}",
            if self.release {"r"} else {""},
            if self.beta    {"b"} else {""},
            if self.alpha   {"a"} else {""},
        )
    }
}

impl BitOr for ReleaseTypeMode {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            release: self.release | rhs.release,
            beta:    self.beta    | rhs.beta,
            alpha:   self.alpha   | rhs.alpha,
        }
    }
}

impl Conf {
    pub fn filter_addon_file(&self, file: &AddonFile, legacy_blacklist: Option<&str>, positive_negative_in_filename: bool) -> bool {
        if !self.game_version.matches(file.game_version.iter()) {
            return false;
        }

        fn in_list(file: &AddonFile, entry: &str) -> bool {
            let entry = entry.trim().to_lowercase();
            for file_game_version in &file.game_version {
                if file_game_version.0.trim().to_lowercase().contains(&entry) {
                    return true;
                }
            }
            false
        }

        fn in_filename(file: &AddonFile, entry: &str) -> bool {
            file.file_name.trim().to_lowercase().contains(&*entry.trim().to_lowercase())
        }

        if self.positive_loader_filter.iter().any(|filter| in_list(file, filter) ) {
            return true;
        }

        if legacy_blacklist.map_or(false, |b|in_list(file, b) ) {
            return false;
        }

        if self.negative_loader_filter.iter().any(|filter| in_list(file, filter) ) {
            return false;
        }

        if positive_negative_in_filename {
            if self.positive_loader_filter.iter().any(|filter| in_filename(file, filter) ) {
                return true;
            }
    
            if legacy_blacklist.map_or(false, |b| in_filename(file, b) ) {
                return false;
            }
    
            if self.negative_loader_filter.iter().any(|filter| in_filename(file, filter) ) {
                return false;
            }
        }

        true
    }
}