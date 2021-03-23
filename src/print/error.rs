use termion::style;

use crate::addon::local::LocalAddons;
use crate::addon::{AddonID, GameVersion};
use crate::api::AddonInfo;
use crate::error;
use crate::hard_error;
use crate::util::match_str::Match;
use crate::print::addons::print_addons_search;

pub fn unwrap_match<T>(r: Result<Match<T>,Vec<Match<T>>>) -> Match<T> {
    match r {
        Ok(r) => r,
        Err(e) if e.is_empty() => hard_error!("Not match for installed addon"),
        Err(e) => {
            error!("Ambiguous matches for installed addon");
            for m in e {
                m.print_error();
            }
            std::process::exit(1);
        }
    }
}
pub fn unwrap_addon_info(r: Result<AddonInfo,Vec<AddonInfo>>, game_version: &GameVersion, installed: &LocalAddons) -> AddonInfo {
    match r {
        Ok(r) => r,
        Err(e) if e.is_empty() => hard_error!("Not match for addon"),
        Err(e) => {
            error!("Ambiguous matches for addon:");
            print_addons_search(&e,game_version,installed);
            error!("Ambiguous matches for addon");
            std::process::exit(1);
        }
    }
}