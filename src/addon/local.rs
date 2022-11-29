use std::collections::HashMap;
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
    #[serde(default = "default_positive_negative_in_filename")]
    pub positive_negative_in_filename: bool,
    pub installed: Option<AddonFile>,
}

fn default_positive_negative_in_filename() -> bool {
    true
}

use rustc_hash::FxHashMap;
use serde::de::{SeqAccess, Visitor};
use clap::Subcommand;

#[derive(Subcommand,Clone,Copy)]
#[derive(Deserialize,Serialize)]
pub enum UpdateOpt {
    /// Update on implicit and update-all
    #[command()]
    All,
    /// Update on implicit, but not on update-all
    #[command()]
    Implicit,
    /// Only update on explicit
    #[command()]
    Explicit,
}

#[repr(transparent)]
pub struct LocalAddons(pub FxHashMap<AddonID,LocalAddon>);

impl Deref for LocalAddons {
    type Target = FxHashMap<AddonID,LocalAddon>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for LocalAddons {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Serialize for LocalAddons {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        
        let mut v: Vec<&LocalAddon> = self.values().collect();
        v.sort_unstable_by_key(|v| &v.slug.0 );
        
        serializer.collect_seq(v)
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
                let mut addons = HashMap::with_capacity_and_hasher(256,Default::default());
    
                while let Some(value) = seq.next_element::<LocalAddon>()? {
                    addons.insert(value.id, value);
                }
    
                Ok(LocalAddons(addons))
            }
        }

        deserializer.deserialize_seq(LAVisitor)
    }
}
