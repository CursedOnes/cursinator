use std::fmt::Display;

use structopt::*;

use crate::addon::local::{LocalAddons, UpdateOpt};
use crate::conf::Conf;
use crate::util::match_str::{self, find_installed_mod_by_key};
use crate::print::error::unwrap_addon_match;
use crate::{error,hard_error};
use crate::util::match_str::match_str;
use super::match_bool;

pub fn gset(conf: &mut Conf, key: Option<&str>, value: Option<&str>) -> bool {
    if let Some(key) = key {
        match match_key(key) {
            WhatGSet::UpdateOpt => if let Some(value) = value {
                addon.update_opt = match_updateopt(value);
            } else {
                eprintln!("\tupdate-opt={}",addon.update_opt);
            },
            WhatGSet::ManuallyInstalled => if let Some(value) = value {
                addon.manually_installed = match_bool(value,"manually-installed");
            } else {
                eprintln!("\tmanually-installed={}",addon.manually_installed);
            },
            WhatGSet::VersionBlacklist => if let Some(value) = value {
                addon.version_blacklist = Some(value.to_owned()); //TODO set none
            } else {
                eprintln!("\tversion-blacklist={}",addon.version_blacklist.as_ref().map(|s| s as &str).unwrap_or(""));
            },
        }
    }else{
        eprintln!(
            "\tupdate-opt={}\n\tmanually-installed={}\n\tversion-blacklist={}",
            addon.update_opt,
            addon.manually_installed,
            addon.version_blacklist.as_ref().map(|s| s as &str).unwrap_or(""),
        );
    }

    todo!()
}

fn match_key(s: &str) -> WhatASet {
    let to_match = &[
        (WhatGSet::UpdateOpt,"update-opt"),
        (WhatGSet::ManuallyInstalled,"manually-installed"),
        (WhatGSet::VersionBlacklist,"version-blacklist"),
    ];
    match match_str(s,||to_match.iter().cloned()) {
        Ok(r) => r.z,
        Err(e) if e.is_empty() => hard_error!("Not match for setting"),
        Err(e) => {
            error!("Ambiguous matches for setting");
            for m in e {
                error!("\t{}{}{}{}{}",m.prefix(),Fg(Blue),m.marked(),Fg(Reset),m.suffix());
            }
            std::process::exit(1);
        }
    }
}

#[derive(Clone)]
enum WhatGSet {
    UrlTxt,
    AddonMtime,
    SoftRetries,
}

fn match_updateopt(s: &str) -> UpdateOpt {
    let to_match = &[
        (UpdateOpt::All,"all"),
        (UpdateOpt::Explicit,"explicit"),
        (UpdateOpt::Implicit,"implicit"),
    ];
    match match_str(s,||to_match.iter().cloned()) {
        Ok(r) => r.z,
        Err(e) if e.is_empty() => hard_error!("Not match for UpdateOpt"),
        Err(e) => {
            error!("Ambiguous matches for UpdateOpt");
            for m in e {
                error!("\t{}{}{}{}{}",m.prefix(),Fg(Blue),m.marked(),Fg(Reset),m.suffix());
            }
            std::process::exit(1);
        }
    }
}
