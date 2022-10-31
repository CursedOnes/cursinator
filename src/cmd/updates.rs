use crate::addon::local::LocalAddon;
use crate::op::update::{find_version_update, fix_discrepancy};
use crate::print::addons::print_addon;
use crate::{Op, error, hard_error, unwrap_result_error};
use crate::addon::rtm::ReleaseTypeMode;
use crate::api::API;
use crate::api::files::FilesResult;
use crate::conf::Repo;
use crate::print::error::unwrap_match;
use crate::print::versions::print_versions;
use crate::util::match_str::find_installed_mod_by_key;
use crate::print::{Koller, term_w, term_h};

pub fn main(
    _: &Op,
    api: &API,
    repo: &Repo,
    rt: Option<ReleaseTypeMode>,
    show_all: bool,
    list_older: bool,
    addon: Option<String>,
) -> bool {
    if let Some(addon) = addon {
        let addon_id = unwrap_result_error!(unwrap_match(find_installed_mod_by_key(&addon,&repo.addons,false/*TODO true*/))).z;

        let addon = &repo.addons.get(&addon_id).unwrap();

        let mut versions = match api.files(addon_id) {
            FilesResult::Ok(f) => f,
            FilesResult::NotFound => hard_error!("No online information for installed addon"),
            FilesResult::Error(e) => hard_error!("Failed to fetch online information: {}",e),
        };

        fix_discrepancy(&mut versions, addon.installed.as_ref().unwrap());

        if !versions.iter().any(|v| repo.conf.game_version.matches(v.game_version.iter()) ) {
            hard_error!("No version for current game version");
        }

        print_versions(
            &versions,
            Some(addon.installed.as_ref().unwrap()),
            rt.unwrap_or(addon.channel),
            &repo.conf.game_version,
            addon.version_blacklist.as_deref(),
            list_older,
            if show_all {16384} else {term_h().saturating_sub(4).max(16) as usize},
        );
    } else {
        let mut addons: Vec<&LocalAddon> = repo.addons.values().collect();
        addons.sort_unstable_by_key(|a| &a.slug.0 );
        for a in addons {
            let installed = match &a.installed {
                Some(h) => h,
                None => continue,
            };

            let mut versions = match api.files(a.id) {
                FilesResult::Ok(f) => f,
                FilesResult::NotFound => {error!("No online information for installed addon");continue},
                FilesResult::Error(e) => {error!("Failed to fetch online information: {}",e);continue},
            };

            fix_discrepancy(&mut versions, installed);

            if !versions.iter().any(|v| repo.conf.game_version.matches(v.game_version.iter()) ) {
                error!("No version for current game version: {}",a.slug);
            }

            let new = find_version_update(
                &versions,
                Some(installed.id),
                &repo.conf.game_version,
                a.version_blacklist.as_deref(),
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
                    term_w() as usize,
                    if show_all {Koller::blue_bold()} else {Default::default()},
                );
            } else if show_all {
                print_addon(
                    &a.slug,
                    &a.name,
                    "",
                    None,
                    Some(installed.release_type),
                    term_w() as usize,
                    Default::default(),
                );
            }
        }
    }
    false
}
