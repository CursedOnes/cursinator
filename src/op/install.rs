use crate::addon::AddonID;
use crate::addon::files::AddonFile;
use crate::addon::local::{LocalAddons, UpdateOpt};
use crate::addon::rtm::ReleaseTypeMode;

pub fn install_mod(
    addons: &mut LocalAddons,
    // install this specific AddonFile
    addon_id: AddonID,
    install: AddonFile,
    // write back to LocalAddon
    channel: ReleaseTypeMode,
    update_opt: UpdateOpt,
    manually_installed: bool,
    version_blacklist: Option<String>,
){
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
}
