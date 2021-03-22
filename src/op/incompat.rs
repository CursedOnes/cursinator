use rustc_hash::FxHashSet;

use crate::addon::AddonID;
use crate::addon::files::AddonFile;
use crate::addon::local::{LocalAddon, LocalAddons};

pub fn check_incompatibility(file: &AddonFile, addon: AddonID, addons: &LocalAddons) -> FxHashSet<AddonID> { //TODO deduped HashSet
    check_incompatibility_2(&[(file,addon)], addons)
}
pub fn check_incompatibility_2(ours: &[(&AddonFile,AddonID)], addons: &LocalAddons) -> FxHashSet<AddonID> { //TODO deduped HashSet
    let mut dest = FxHashSet::default();
    for (file,addon) in ours {
        for other_addon in addons.values()  {
            if other_addon.id != *addon {
                if let Some(other_file) = &other_addon.installed {
                    if 
                        file.dependencies.iter_incompatible().find(|i| *i == other_addon.id ).is_some() ||
                        other_file.dependencies.iter_incompatible().find(|i| i == addon ).is_some()
                    {
                        dest.insert(other_addon.id);
                    }
                }
            }
        }
        for (other_file,other_addon) in ours {
            if other_addon != addon {
                if 
                    file.dependencies.iter_incompatible().find(|i| i == other_addon ).is_some() ||
                    other_file.dependencies.iter_incompatible().find(|i| i == addon ).is_some()
                {
                    dest.insert(*other_addon);
                }
            }
        }
    }
    dest
}
