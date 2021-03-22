use termion::style;

use crate::addon::AddonID;
use crate::error;
use crate::hard_error;
use crate::util::match_str::Match;

pub fn unwrap_addon_match(r: Result<Match<AddonID>,Vec<Match<AddonID>>>) -> Match<AddonID> {
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
