use crate::{Op, unwrap_result_error, log_error, error};
use crate::addon::rtm::ReleaseTypeMode;
use crate::api::API;
use crate::conf::Repo;
use crate::hard_error;
use crate::api::files::FilesResult;
use crate::util::match_str::*;
use crate::print::error::unwrap_match;
use crate::op::update::{find_version_update, fix_discrepancy};
use crate::op::install::install_mod;
use crate::addon::local::UpdateOpt;

pub fn main(
    o: &Op,
    api: &mut API,
    repo: &mut Repo,
    rt: Option<ReleaseTypeMode>,
    allow_downgrade: bool,
    force: bool,
    addon: String,
    version: Option<String>,
) -> bool {
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

    if !versions.iter().any(|v| repo.conf.filter_addon_file(v, addon.version_blacklist.as_deref(), addon.positive_negative_in_filename) ) {
        hard_error!("No version for current filter");
    }

    let channel = rt.unwrap_or(addon.channel); //TODO use channel from previous install

    let file;
    if let Some(version) = version {
        //TODO detect if version is a file id
        file = unwrap_result_error!(unwrap_match(find_to_install_version_by_key( &version, &versions,&repo.conf.game_version))).z;
        if let Some(i) = addon.installed.as_ref() {
            if file.id.0 < i.id.0 && !allow_downgrade {
                hard_error!("Not downgrading");
            }
        }
    } else {
        if let UpdateOpt::Explicit = addon.update_opt {
            hard_error!("Addon update rule is set to explicit");
        }

        let new = find_version_update(
            &versions,
            Some(addon.installed.as_ref().unwrap().id),
            &repo.conf,
            addon.version_blacklist.as_deref(),
            addon.positive_negative_in_filename,
            channel,
            allow_downgrade,
        );
        match new {
            Some(a) => file = a,
            None => hard_error!("No version found to update to"),
        }
    }

    let result = install_mod(
        addon.id,
        file.clone(),
        force,
        addon.slug.clone(),
        addon.name.clone(),
        channel,
        addon.update_opt, //TODO give as arg
        addon.manually_installed,
        addon.version_blacklist.clone(), //TODO give vb as arg
        addon.positive_negative_in_filename,
        o,
        api,
        repo,
    );

    match result {
        Ok(v) => true,
        Err(e) =>  {error!("Error updating mod: {}",e);false},
    }
}
