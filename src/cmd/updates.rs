use crate::{Op, hard_error};
use crate::addon::release_type::ReleaseType;
use crate::addon::rtm::ReleaseTypeMode;
use crate::api::API;
use crate::api::files::FilesResult;
use crate::conf::Repo;
use crate::print::error::unwrap_addon_match;
use crate::print::versions::print_versions;
use crate::util::match_str::find_installed_mod_by_key;
use crate::unwrap_result_error;
use crate::print::term_h;

pub fn main(
    o: &Op,
    api: &API,
    repo: &Repo,
    rt: Option<ReleaseTypeMode>,
    show_all: bool,
    list_older: bool,
    addon: Option<String>,
) -> bool {
    if let Some(addon) = addon {
        let addon_id = unwrap_addon_match(find_installed_mod_by_key(&addon,&repo.addons)).z;

        let addon = &repo.addons.get(&addon_id).unwrap();

        let versions = match api.files(addon_id) {
            FilesResult::Ok(f) => f,
            FilesResult::NotFound => hard_error!("No online information for installed addon"),
            FilesResult::Error(e) => hard_error!("Failed to fetch online information: {}",e),
        };

        print_versions(
            &versions,
            Some(addon.installed.as_ref().unwrap()),
            rt.unwrap_or(addon.channel),
            &repo.conf.game_version,
            list_older,
            if show_all {16384} else {term_h().saturating_sub(4) as usize},
        );
    } else {
        todo!();
    }
    false
}
