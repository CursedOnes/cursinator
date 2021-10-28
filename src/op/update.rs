use crate::addon::{FileID, GameVersion};
use crate::addon::files::AddonFile;
use crate::addon::rtm::ReleaseTypeMode;
use crate::warn;

pub fn find_version_update<'a>(
    versions: &'a [AddonFile],
    installed: Option<FileID>,
    game_version: &GameVersion,
    blacklist: Option<&str>,
    release_type: ReleaseTypeMode,
    allow_downgrade: bool,
) -> Option<&'a AddonFile> {
    let mut current_idx = 0; // includes current version
    if let Some(installed) = installed {
        for v in versions {
            if v.id.0 < installed.0 {
                current_idx += 1;
            } else {
                break
            }
        }
    } else {
        current_idx = versions.len();
    }

    let visible_range = if allow_downgrade || installed.is_none() {
        0..versions.len()
    } else {
        current_idx..versions.len()
    };

    release_type.pick_version(&versions[visible_range], game_version, blacklist)
        .filter(|f| installed.is_none() || f.id != installed.unwrap() )
}

pub fn fix_discrepancy(
    versions: &mut Vec<AddonFile>,
    installed: &AddonFile,
) {
    let remote = versions.iter()
        .find(|v| v.id == installed.id );

    if let Some(remote) = remote {
        if remote.release_type != installed.release_type {
            warn!(
                "{}: Release type discrepancy between local and online info: Perhaps the Release type of the file was modified online after installation",
                installed.file_name,
            );
        }

        if installed.dependencies.new_required(&remote.dependencies) {
            warn!(
                "{}: Discrepancy in Required Dependencies: Perhaps the Addon needs to be updated or reinstalled",
                installed.file_name,
            );
        }
    } else {
        warn!(
            "{}: Currently installed version not available online",
            installed.file_name,
        );
        // substitute version info from installed

        // find the next slot after the last with smaller id
        let slot = versions.iter()
            .enumerate()
            .find(|(_,v)| v.id.0 > installed.id.0 )
            .map(|(i,_)| i )
            .unwrap_or(versions.len());

        versions.insert(slot, installed.clone());
    }
}
