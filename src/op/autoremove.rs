use rustc_hash::FxHashSet;

use crate::addon::AddonID;
use crate::addon::local::{LocalAddon, LocalAddons};

pub fn autoremovable(addons: &LocalAddons) -> Vec<AddonID> {
    let mut has_depedents = FxHashSet::default();
    for addon in addons.values() {
        if let Some(file) = &addon.installed {
            for deps in file.dependencies.iter_required() {
                has_depedents.insert(deps);
            }
        }
    }

    let mut dest = Vec::with_capacity(addons.len()-has_depedents.len());
    for addon in addons.values() {
        if !addon.manually_installed && has_depedents.contains(&addon.id) {
            dest.push(addon.id);
        }
    }
    
    dest
}
