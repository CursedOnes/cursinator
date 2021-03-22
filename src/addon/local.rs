use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use super::*;
use super::files::AddonFile;
use super::rtm::ReleaseTypeMode;

#[derive(Deserialize,Serialize)]
pub struct LocalAddon { //TODO defaults
    pub id: AddonID,
    pub slug: AddonSlug,
    pub name: String,
    pub channel: ReleaseTypeMode,
    pub update_opt: UpdateOpt,
    pub manually_installed: bool,
    pub version_blacklist: Option<String>, //blacklist versions with occurrence in game versions or filename
    pub installed: Option<AddonFile>,
}

use chrono::Local;
use rustc_hash::FxHashMap;
use serde::de::{SeqAccess, Visitor};
use structopt::*;

#[derive(StructOpt,Clone)]
#[derive(Deserialize,Serialize)]
pub enum UpdateOpt {
    /// update on implicit and update-all
    #[structopt(about="Update on implicit and update-all")]
    All,
    /// update on implicit, but not on update-all
    #[structopt(about="Update on implicit, but not on update-all")]
    Implicit,
    /// only update on explicit
    #[structopt(about="Only update on explicit")]
    Explicit,
}

#[repr(transparent)]
pub struct LocalAddons {
    addons: FxHashMap<AddonID,LocalAddon>,
}

impl Deref for LocalAddons {
    type Target = FxHashMap<AddonID,LocalAddon>;

    fn deref(&self) -> &Self::Target {
        &self.addons
    }
}
impl DerefMut for LocalAddons {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.addons
    }
}

impl Serialize for LocalAddons {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        
        serializer.collect_seq(self.addons.values())
    }
}
impl<'de> Deserialize<'de> for LocalAddons {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        
        struct LAVisitor;
    
        impl<'de> Visitor<'de> for LAVisitor {
            type Value = LocalAddons;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence")
            }
    
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut addons = FxHashMap::default();
    
                while let Some(value) = seq.next_element::<LocalAddon>()? {
                    addons.insert(value.id, value);
                }
    
                Ok(LocalAddons{addons})
            }
        }

        deserializer.deserialize_seq(LAVisitor)
    }
}
