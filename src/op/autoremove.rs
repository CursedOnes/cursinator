use rustc_hash::FxHashSet;

use crate::addon::AddonID;
use crate::addon::local::LocalAddons;

pub fn autoremovable(addons: &LocalAddons) -> Vec<AddonID> {
    // collect addons which have dependents (are dep in other addons)
    let mut has_dependents = FxHashSet::default();
    for addon in addons.values() {
        if let Some(file) = &addon.installed {
            for deps in file.dependencies.iter_required() {
                has_dependents.insert(deps);
            }
        }
    }

    // collect auto-installed addons without dependents
    let mut dest = Vec::with_capacity(addons.len()-has_dependents.len());
    for addon in addons.values() {
        if addon.installed.is_some() && !addon.manually_installed && !has_dependents.contains(&addon.id) {
            dest.push(addon.id);
        }
    }
    
    dest
}
