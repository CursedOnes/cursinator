use rustc_hash::FxHashSet;

use crate::addon::local::{LocalAddon, LocalAddons};

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
