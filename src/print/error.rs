use anyhow::bail;

use crate::addon::local::LocalAddons;
use crate::addon::GameVersion;
use crate::api::AddonInfo;
use crate::error;
use crate::hard_error;
use crate::util::match_str::Match;
use crate::print::addons::print_addons_search;

pub fn unwrap_match<T>(r: Result<Match<T>,Vec<Match<T>>>) -> Result<Match<T>,anyhow::Error> {
    match r {
        Ok(r) => Ok(r),
        Err(e) if e.is_empty() => Err(anyhow::anyhow!("No match for installed addon")),
        Err(e) => {
            let mut error_message = "Ambiguous matches for installed addon".to_owned();
            for m in e {
                error_message += &m.fmt_error("\n");
            }
            Err(anyhow::anyhow!(error_message))
        }
    }
}
pub fn unwrap_addon_info(r: Result<AddonInfo,Vec<AddonInfo>>, game_version: &GameVersion, installed: &LocalAddons) -> AddonInfo {
    match r {
        Ok(r) => r,
        Err(e) if e.is_empty() => hard_error!("No match for addon"),
        Err(e) => {
            error!("Ambiguous matches for addon:");
            print_addons_search(e.iter(),game_version,installed);
            error!("Ambiguous matches for addon");
            std::process::exit(1);
        }
    }
}