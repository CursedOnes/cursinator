use crate::addon::{AddonID, AddonSlug, GameVersion};
use crate::addon::files::AddonFile;
use crate::addon::local::{LocalAddon, LocalAddons, UpdateOpt};
use crate::addon::rtm::ReleaseTypeMode;
use crate::api::API;
use crate::conf::Repo;
use super::deps::collect_deps;
use super::incompat::*;
use crate::{Op, error, warn, unwrap_result_error};

pub fn install_mod(
    // install this specific AddonFile
    addon_id: AddonID,
    install: AddonFile,
    force_incompat: bool, //install even if incompat
    // write back to LocalAddon
    i_slug: AddonSlug,
    i_name: String,
    channel: ReleaseTypeMode,
    update_opt: UpdateOpt,
    manually_installed: bool,
    version_blacklist: Option<String>,
    // oof
    o: &Op,
    api: &API,
    repo: &mut Repo,
) -> bool {
    // if current mod installed, add to delete_sched
    // iterate required deps of to install file recursively and if not already installed, collect to install_sched, choose the latest version matching channel
    // - only deps that aren't installed are now installed
    // - TODO what if dep's LocalAddon still exists?
    // do check_incompatibility_2 with install_sched
    // now handle --force to override incompatibilities and noop
    // attempt to install all addons in install_sched, collect finalizers
    // - on deps will softly derive from "our" LocalParams and existing LocalAddon (if removed but not purged) TODO how
    // - on "this" addon, "our" LocalParams will replace the ones of existing LocalAddon
    // run install finalizers and delete_sched

    let mut install_queue = vec![];
    let mut finalizer_queue = vec![];

    collect_deps(
        &repo.addons,
        api,
        install.dependencies.iter_required(),
        &repo.conf.game_version,
        channel,
        update_opt,
        &version_blacklist,
        &mut install_queue,
    );

    let incompat = check_incompatibility_3(
        &install_queue,
        &repo.addons,
    );

    if !incompat.is_empty() {
        if !force_incompat {
            error!("Incompatible addons:{}",o.suffix());
        } else {
            warn!("Installing addons with incompatibilities:{}",o.suffix());
        }

        for i in incompat {
            eprint!("    {} => {}",i.from.slug,i.to.slug);
        }
        eprintln!("{}",o.suffix());

        if !force_incompat {
            std::process::exit(1);
        }
    }

    let mut modified = false;

    for i in install_queue {
        eprintln!("Install {}{}",i.slug,o.suffix());
        if !o.noop {
            let finalizer = i.installed.as_ref().unwrap().download(&repo.conf,api);
            finalizer_queue.push(finalizer);
            repo.addons.insert(i.id,i);
            modified = true;
        }
    }

    eprintln!("Install {}{}",i_slug,o.suffix());
    if !o.noop {
        let finalizer = install.download(&repo.conf,api);
        finalizer_queue.push(finalizer);
    
        let new = LocalAddon {
            id: addon_id,
            slug: i_slug,
            name: i_name,
            channel,
            update_opt,
            manually_installed,
            version_blacklist,
            installed: Some(install),
        };

        repo.addons.insert(addon_id,new);

        modified = true;
    }

    if let Some(addon) = repo.addons.get_mut(&addon_id) {
        if let Some(installed) = &mut addon.installed {
            eprintln!("Uninstall previous version of {}{}",addon.slug,o.suffix());
            if !o.noop {
                unwrap_result_error!(installed.remove(),|e|"Failed to remove addon: {}",e);
                modified = true;
            }
        }
        if !o.noop {
            addon.installed = None;
            modified = true;
        }
    }

    drop(finalizer_queue);

    modified
}
