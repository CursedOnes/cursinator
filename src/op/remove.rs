use crate::addon::AddonID;
use crate::addon::local::{LocalAddon, LocalAddons};

pub fn has_dependents(id: AddonID, addons: &LocalAddons) -> Vec<&LocalAddon> {
    let mut dest = Vec::new();

    for addon in addons.values() {
        if let Some(file) = &addon.installed {
            for deps in file.dependencies.iter_required() {
                if deps == id {
                    dest.push(addon);
                }
            }
        }
    }
    
    dest
}
