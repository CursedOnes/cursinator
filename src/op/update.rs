use crate::addon::{FileID, GameVersion};
use crate::addon::files::AddonFile;
use crate::addon::rtm::ReleaseTypeMode;

pub fn find_version_update<'a>(
    versions: &'a [AddonFile],
    installed: Option<FileID>,
    game_version: &GameVersion,
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

    release_type.pick_version_2(&versions[visible_range], game_version)
        .filter(|f| installed.is_none() || f.id != installed.unwrap() )
}
