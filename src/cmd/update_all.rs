use std::collections::HashMap;

use crate::Op;
use crate::addon::local::UpdateOpt;
use crate::addon::rtm::ReleaseTypeMode;
use crate::api::API;
use crate::api::files::FilesResult;
use crate::conf::Repo;
use crate::error;
use crate::op::install::install_mod;
use crate::op::update::{find_version_update, fix_discrepancy};

pub fn main(
    o: &Op,
    api: &mut API,
    repo: &mut Repo,
    rt: Option<ReleaseTypeMode>,
) -> bool {

    let mut cache = HashMap::with_capacity_and_hasher(256,Default::default());

    let mut modified = false;

    loop {
        let mut repeat = false;

        let mut queue= Vec::new();

        for addon in repo.addons.values() {
            match addon.update_opt {
                UpdateOpt::All => {},
                _ => continue,
            }

            let installed = match &addon.installed {
                Some(h) => h,
                None => continue,
            };

            let mut versions = match api.files_cached(addon.id,&mut cache) {
                FilesResult::Ok(f) => f,
                FilesResult::NotFound => {error!("No online information for installed addon");continue},
                FilesResult::Error(e) => {error!("Failed to fetch online information: {}",e);continue},
            };

            fix_discrepancy(&mut versions, installed);

            if !versions.iter().any(|v| repo.conf.game_version.matches(v.game_version.iter()) ) {
                error!("No version for current game version: {}",addon.slug);
            }

            let new = find_version_update(
                &versions,
                Some(installed.id),
                &repo.conf.game_version,
                addon.version_blacklist.as_deref(),
                addon.channel, //TODO channel arg
                false, //TODO allow_upgrade arg
            );

            if let Some(new) = new {
                queue.push((
                    addon.id,
                    addon.slug.clone(),
                    addon.name.clone(),
                    addon.channel,
                    addon.update_opt,
                    addon.manually_installed,
                    addon.version_blacklist.clone(),
                    new.clone(),
                ));
            }
        }

        for (id,slug,name,channel,update_opt,manually_installed,version_blacklist,file) in queue {
            let result = install_mod(
                id,
                file,
                false,
                slug,
                name,
                channel, //TODO channel arg
                update_opt, //TODO give as arg
                manually_installed,
                version_blacklist, //TODO give vb as arg
                o,
                api,
                repo,
            );
            
            match result {
                Ok(v) => repeat |= v,
                Err(e) => error!("Error updating mod: {}",e),
            }

            modified |= repeat;
        }

        if !repeat {
            break
        }
    }
    
    modified
}
