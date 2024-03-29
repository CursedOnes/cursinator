use std::fmt::Display;

use crate::addon::local::UpdateOpt;
use crate::conf::Repo;
use crate::util::match_str::find_installed_mod_by_key;
use crate::print::error::unwrap_match;
use crate::{Op, error, hard_error, unwrap_result_error};
use crate::util::match_str::match_str;
use super::match_bool;

pub fn main(
    o: &Op,
    repo: &mut Repo,
    addon: String,
    key: Option<String>,
    value: Option<String>
) -> bool {
    let addons = &mut repo.addons;
    let addon_id = unwrap_result_error!(unwrap_match(find_installed_mod_by_key(&addon,addons,true))).z;
    let addon = addons.get_mut(&addon_id).unwrap();

    if let Some(key) = key {
        match match_key(&key) {
            WhatASet::UpdateOpt => if let Some(value) = value {
                if o.noop {return false;}
                addon.update_opt = match_updateopt(&value);
                true
            } else {
                eprintln!("\tupdate-opt={}",addon.update_opt);
                false
            },
            WhatASet::ManuallyInstalled => if let Some(value) = value {
                if o.noop {return false;}
                addon.manually_installed = match_bool(&value,"manually-installed");
                true
            } else {
                eprintln!("\tmanually-installed={}",addon.manually_installed);
                false
            },
            WhatASet::VersionBlacklist => if let Some(value) = value {
                if o.noop {return false;}
                if value == "-" {
                    addon.version_blacklist = None;
                } else {
                    addon.version_blacklist = Some(value);
                }
                true
            } else {
                eprintln!("\tversion-blacklist={}",addon.version_blacklist.as_ref().map(|s| s as &str).unwrap_or(""));
                false
            },
        }
    }else{
        eprintln!(
            "\tupdate-opt={}\n\tmanually-installed={}\n\tversion-blacklist={}\n\nNot all options exposed yet, refer repo.json",
            addon.update_opt,
            addon.manually_installed,
            addon.version_blacklist.as_ref().map(|s| s as &str).unwrap_or(""),
        );
        false
    }
}

fn match_key(s: &str) -> WhatASet {
    let to_match = [&[
        (WhatASet::UpdateOpt,"update-opt"),
        (WhatASet::ManuallyInstalled,"manually-installed"),
        (WhatASet::VersionBlacklist,"version-blacklist"),
    ][..]];
    match match_str(s,to_match) {
        Ok(r) => r.z,
        Err(e) if e.is_empty() => hard_error!("No match for setting"),
        Err(e) => {
            error!("Ambiguous matches for setting");
            for m in e {
                m.print_error();
            }
            std::process::exit(1);
        }
    }
}

#[derive(Clone)]
enum WhatASet {
    UpdateOpt,
    ManuallyInstalled,
    VersionBlacklist,
}

fn match_updateopt(s: &str) -> UpdateOpt {
    let to_match = [&[
        (UpdateOpt::All,"all"),
        (UpdateOpt::Explicit,"explicit"),
        (UpdateOpt::Implicit,"implicit"),
    ][..]];
    match match_str(s,to_match) {
        Ok(r) => r.z,
        Err(e) if e.is_empty() => hard_error!("No match for UpdateOpt"),
        Err(e) => {
            error!("Ambiguous matches for UpdateOpt");
            for m in e {
                m.print_error();
            }
            std::process::exit(1);
        }
    }
}

impl Display for UpdateOpt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = match self {
            UpdateOpt::All => "all",
            UpdateOpt::Implicit => "implicit",
            UpdateOpt::Explicit => "explicit",
        };
        write!(f,"{v}")
    }
}
