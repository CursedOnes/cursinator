use crate::addon::rtm::ReleaseTypeMode;
use crate::addon::{AddonID, GameVersion};
use crate::addon::files::AddonFile;
use crate::addon::local::{LocalAddon, LocalAddons, UpdateOpt};
use crate::api::API;
use crate::hard_error;
use crate::unwrap_or_error;
use crate::api::files::FilesResult;

pub fn collect_deps(
    installed: &LocalAddons,
    api: &API,
    deps: impl Iterator<Item=AddonID>,
    install_queue: &mut Vec<LocalAddon>,
    game_version: &GameVersion,
    channel: ReleaseTypeMode,
    update_opt: UpdateOpt,
    version_blacklist: &Option<String>,
) {
    // version picking for deps:
    // 1. if explicit version for parent, filter to-install dep versions before specific date
    //for dep in 

    for dep_id in deps {
        if installed.contains_key(&dep_id) {continue}

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
            Ok(None) => hard_error!("Dependency not available"),
            Err(e) => hard_error!("Failed to fetch dependency: {}",e),
        };
        let dep_files = match api.files(dep_id) {
            FilesResult::Ok(v) => v,
            FilesResult::NotFound => hard_error!("Dependency not available"),
            FilesResult::Error(e) => hard_error!("Failed to fetch dependency: {}",e),
        };

        let mut dep_files: Vec<_> = dep_files.into_iter()
            .filter(|f| game_version.matches(f.game_version.iter()) )
            .collect();
        let file_idx = z_channel.pick_version(&dep_files); //TODO do blacklist
        let file_idx = unwrap_or_error!(file_idx,"H");

        let dep_file = dep_files.swap_remove(file_idx);

        collect_deps(
            installed,
            api,
            dep_file.dependencies.iter_required(),
            install_queue,
            game_version,
            z_channel, //TODO z or non-z?
            z_update_opt,
            &z_version_blacklist,
        );

        let new_dep = LocalAddon {
            id: dep_id,
            slug: dep_info.slug,
            name: dep_info.name,
            channel: z_channel,
            update_opt: z_update_opt,
            manually_installed: z_manually_installed,
            version_blacklist: z_version_blacklist,
            installed: Some(dep_file),
        };

        install_queue.push(new_dep);
    }
}
