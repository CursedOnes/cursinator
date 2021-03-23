use crate::op::update::find_version_update;
use crate::print::addons::print_addon;
use crate::{Op, error, hard_error};
use crate::addon::release_type::ReleaseType;
use crate::addon::rtm::ReleaseTypeMode;
use crate::api::API;
use crate::api::files::FilesResult;
use crate::conf::Repo;
use crate::print::error::unwrap_match;
use crate::print::versions::print_versions;
use crate::util::match_str::find_installed_mod_by_key;
use crate::unwrap_result_error;
use crate::print::{Koller, term_h};

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
        let addon_id = unwrap_match(find_installed_mod_by_key(&addon,&repo.addons,false/*TODO true*/)).z;

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
        for a in repo.addons.values() {
            let installed = match &a.installed {
                Some(h) => h,
                None => continue,
            };

            let versions = match api.files(a.id) {
                FilesResult::Ok(f) => f,
                FilesResult::NotFound => {error!("No online information for installed addon");continue},
                FilesResult::Error(e) => {error!("Failed to fetch online information: {}",e);continue},
            };

            let new = find_version_update(
                &versions,
                Some(installed.id),
                &repo.conf.game_version,
                a.channel,
                list_older,
            );

            if let Some(new) = new {
                print_addon(
                    &a.slug,
                    &a.name,
                    "",
                    Some(new.release_type),
                    Some(installed.release_type),
                    term_h() as usize,
                    if show_all {Koller::blue_bold()} else {Default::default()},
                );
            } else if show_all {
                print_addon(
                    &a.slug,
                    &a.name,
                    "",
                    None,
                    Some(installed.release_type),
                    term_h() as usize,
                    Default::default(),
                );
            }
        }
    }
    false
}