use anyhow::bail;

use crate::addon::rtm::ReleaseTypeMode;
use crate::addon::{AddonID, GameVersion};
use crate::addon::local::{LocalAddon, LocalAddons, UpdateOpt};
use crate::api::API;
use crate::unwrap_or_bail;
use crate::api::files::FilesResult;

pub fn collect_deps(
    installed: &LocalAddons,
    api: &mut API,
    deps: impl Iterator<Item=AddonID>,
    game_version: &GameVersion,
    channel: ReleaseTypeMode,
    update_opt: UpdateOpt,
    version_blacklist: &Option<String>,
    install_queue: &mut Vec<LocalAddon>,
) -> Result<(),anyhow::Error> {
    // version picking for deps:
    // 1. if explicit version for parent, filter to-install dep versions before specific date
    //for dep in 

    for dep_id in deps {
        if installed.contains_key(&dep_id) {continue}
        if install_queue.iter().any(|a| a.id == dep_id ) {continue}

        let mut z_channel = channel;
        let mut z_update_opt = update_opt;
        let mut z_manually_installed = false;
        let mut z_version_blacklist = version_blacklist.clone();

        if let Some(local_dep) = installed.get(&dep_id) {
            z_channel = z_channel | local_dep.channel;
            //z_update_opt = local_dep.update_opt; //TODO everything is wrong with the LocalAddon but not installed remove != purge BS, maybe disband it
            z_manually_installed = local_dep.manually_installed;
            z_version_blacklist = local_dep.version_blacklist.clone();
        }

        let dep_info = match api.addon_info(dep_id) {
            Ok(Some(d)) => d,
            Ok(None) => bail!("Dependency not available"),
            Err(e) => bail!("Failed to fetch dependency: {}",e),
        };
        let dep_files = match api.files(dep_id) {
            FilesResult::Ok(v) => v,
            FilesResult::NotFound => bail!("Dependency not available"),
            FilesResult::Error(e) => bail!("Failed to fetch dependency: {}",e),
        };

        if !dep_files.iter().any(|v| game_version.matches(v.game_version.iter()) ) {
            bail!("No version for current game version: {}",dep_info.slug);
        }

        let dep_file = unwrap_or_bail!(
            z_channel.pick_version(
                &dep_files,
                game_version,
                z_version_blacklist.as_deref(),
            ),
            "No version found to install"
        ); //TODO do blacklist

        collect_deps(
            installed,
            api,
            dep_file.dependencies.iter_required(),
            game_version,
            z_channel, //TODO z or non-z?
            z_update_opt,
            &z_version_blacklist,
            install_queue,
        )?;

        let new_dep = LocalAddon {
            id: dep_id,
            slug: dep_info.slug,
            name: dep_info.name,
            channel: z_channel,
            update_opt: z_update_opt,
            manually_installed: z_manually_installed,
            version_blacklist: z_version_blacklist,
            installed: Some(dep_file.clone()),
        };

        install_queue.push(new_dep);
    }

    Ok(())
}
