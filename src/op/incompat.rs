use std::ops::{Deref, DerefMut};

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
pub fn check_incompatibility_3<'a>(ours: &'a [LocalAddon], addons: &'a LocalAddons) -> FxHashSet<Incompat<'a>> { //TODO deduped HashSet
    let mut dest = FxHashSet::default();
    for addon in ours {
        for other_addon in addons.values()  {
            if other_addon.id != addon.id {
                if let Some(file) = &addon.installed {
                    if let Some(other_file) = &other_addon.installed {
                        if 
                            file.dependencies.iter_incompatible().find(|i| *i == other_addon.id ).is_some() ||
                            other_file.dependencies.iter_incompatible().find(|i| *i == addon.id ).is_some()
                        {
                            dest.insert(Incompat{from:addon,to:other_addon});
                        }
                    }
                }
            }
        }
        for other_addon in ours {
            if other_addon.id != addon.id {
                if let Some(file) = &addon.installed {
                    if let Some(other_file) = &other_addon.installed {
                        if 
                            file.dependencies.iter_incompatible().find(|i| *i == other_addon.id ).is_some() ||
                            other_file.dependencies.iter_incompatible().find(|i| *i == addon.id ).is_some()
                        {
                            dest.insert(Incompat{from:addon,to:other_addon});
                        }
                    }
                }
            }
        }
    }
    dest
}

pub struct Incompat<'a> {
    pub from: &'a LocalAddon,
    pub to: &'a LocalAddon,
}

impl PartialEq for Incompat<'_> {
    fn eq(&self, other: &Self) -> bool {
        (self.from.id,self.to.id) == (other.from.id,other.to.id) ||
        (self.from.id,self.to.id) == (other.to.id,other.from.id)
    }
}
impl Eq for Incompat<'_> {}

impl std::hash::Hash for Incompat<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if self.to.id.0 > self.from.id.0 {
            (self.from.id,self.to.id).hash(state)
        } else {
            (self.to.id,self.from.id).hash(state)
        }
    }
}


#[repr(transparent)]
pub struct LocalAddonRef(LocalAddon);

impl LocalAddonRef {
    pub fn from_ref<'a>(r: &'a LocalAddon) -> &'a Self {
        unsafe {
            &*(r as *const LocalAddon as *const LocalAddonRef)
        }
    }
    pub fn from_mut<'a>(r: &'a mut LocalAddon) -> &'a mut Self {
        unsafe {
            &mut *(r as *mut LocalAddon as *mut LocalAddonRef)
        }
    }
}

impl PartialEq for LocalAddonRef {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for LocalAddonRef {}

impl Deref for LocalAddonRef {
    type Target = LocalAddon;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for LocalAddonRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::hash::Hash for LocalAddonRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}
