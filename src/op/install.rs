use anyhow::{bail, anyhow};

use crate::addon::{AddonID, AddonSlug};
use crate::addon::files::AddonFile;
use crate::addon::local::{LocalAddon, UpdateOpt};
use crate::addon::rtm::ReleaseTypeMode;
use crate::api::API;
use crate::conf::Repo;
use crate::util::fs::Finalize;
use super::deps::collect_deps;
use super::incompat::*;
use crate::{Op, error, warn, log_error};

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
    positive_negative_in_filename: bool,
    // oof
    o: &Op,
    api: &mut API,
    repo: &mut Repo,
) -> Result<bool,anyhow::Error> {
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
    let mut installed_queue = vec![];

    collect_deps(
        &repo.addons,
        api,
        install.dependencies.iter_required(),
        &repo.conf,
        channel,
        update_opt,
        &version_blacklist,
        positive_negative_in_filename,
        &mut install_queue,
    )?;

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
            eprintln!("\t{} => {}{}",i.from.slug,i.to.slug,o.suffix());
        }

        if !force_incompat {
            bail!("Incompatible addons:{}",o.suffix());
        }
    }

    let mut modified = false;

    for i in install_queue {
        eprintln!(
            "Install: {} ({}){}",
            i.slug,
            i.installed.as_ref().unwrap().file_name,
            o.suffix()
        );

        if i.installed.as_ref().unwrap().has_install_script {
            warn!(
                "Installing {}: Install Scripts are currently unsupported", 
                i.slug
            );
        }

        if !o.noop {
            let dep_to_install = i.installed.as_ref().unwrap();

            let dep_install_paths = dep_to_install.file_paths_new(
                i.id,
                false,
                &repo.conf,
            );

            let finalizer = dep_to_install.download(&dep_install_paths, &repo.conf, api, false)
                .map_err(|e| anyhow!("Failed to install dependency addon: {}",e))?;

            finalizer_queue.push(finalizer);
            installed_queue.push((i.id,i));
        }
    }

    eprintln!(
        "Install: {} ({}){}",
        i_slug,
        install.file_name,
        o.suffix()
    );

    let prev_paths = repo.addons.get(&addon_id)
        .and_then(|a| a.installed.as_ref() )
        .map(|f| f.file_paths_current(addon_id, !o.noop, &repo.conf) );

    let mut installed_paths = None;

    if !o.noop {
        let install_paths = install.file_paths_new(
            addon_id,
            prev_paths.as_ref().map_or(false, |prev| prev.disabled),
            &repo.conf,
        );

        let finalizer = install.download(&install_paths, &repo.conf, api, false)
            .map_err(|e| anyhow!("Failed to install addon: {}",e))?;

        finalizer_queue.push(finalizer);
    
        installed_queue.push((addon_id,LocalAddon {
            id: addon_id,
            slug: i_slug,
            name: i_name,
            channel,
            update_opt,
            manually_installed,
            version_blacklist,
            positive_negative_in_filename,
            installed: Some(install),
        }));

        installed_paths = Some(install_paths);
    }

    if let Some(installed_paths) = installed_paths {
        if let Some(prev_paths) = prev_paths {
            if installed_paths.path != prev_paths.path {
                eprintln!(
                    "Remove previous version: {}{}",
                    prev_paths.path.to_string_lossy(),
                    o.suffix()
                );
            }
            if !o.noop {
                log_error!(prev_paths.remove_if_not_new(&installed_paths), |e| "Failed to remove addon: {}",e);
                //addon.installed = None; //at that point, the entire install fn succeeded
                modified = true;
            }
        }
    }

    Finalize::finalize_slice(&mut finalizer_queue)?;

    for (id,i) in installed_queue {
        repo.addons.insert(id,i);
        modified = true;
    }

    Ok(modified)
}
