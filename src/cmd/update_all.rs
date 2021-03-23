use crate::Op;
use crate::addon::local::UpdateOpt;
use crate::addon::release_type::ReleaseType;
use crate::addon::rtm::ReleaseTypeMode;
use crate::api::API;
use crate::api::files::FilesResult;
use crate::conf::Repo;
use crate::error;
use crate::op::install::install_mod;
use crate::op::update::find_version_update;

pub fn main(
    o: &Op,
    api: &API,
    repo: &mut Repo,
    rt: Option<ReleaseTypeMode>,
) -> bool {
    let mut modified = false;

    for addon in repo.addons.values() {
        match addon.update_opt {
            UpdateOpt::All => {},
            _ => continue,
        }

        let installed = match &addon.installed {
            Some(h) => h,
            None => continue,
        };

        let versions = match api.files(addon.id) {
            FilesResult::Ok(f) => f,
            FilesResult::NotFound => {error!("No online information for installed addon");continue},
            FilesResult::Error(e) => {error!("Failed to fetch online information: {}",e);continue},
        };

        let new = find_version_update(
            &versions,
            Some(installed.id),
            &repo.conf.game_version,
            addon.channel, //TODO channel arg
            false, //TODO allow_upgrade arg
        );

        if let Some(new) = new {
            todo!()
            /*modified |= install_mod(
                addon.id,
                new.clone(),
                false,
                addon.slug.clone(),
                addon.name.clone(),
                addon.channel, //TODO channel arg
                addon.update_opt, //TODO give as arg
                addon.manually_installed,
                addon.version_blacklist.clone(), //TODO give vb as arg
                o,
                api,
                repo,
            )*/
        }
    }
    
    modified
}
