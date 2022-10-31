use std::convert::TryInto;
use std::ops::{Deref, DerefMut};

use furse::structures::file_structs::{FileDependency, FileRelationType};

use super::*;

#[derive(Clone,PartialEq)]
pub enum Dependency {
    EmbeddedLibrary(AddonID),
    Optional(AddonID),
    Required(AddonID),
    Tool(AddonID),
    Incompatible(AddonID),
    Include(AddonID),
}

impl Dependency {
    pub fn id(&self) -> AddonID {
        match self {
            Dependency::EmbeddedLibrary(i) => *i,
            Dependency::Optional(i)        => *i,
            Dependency::Required(i)        => *i,
            Dependency::Tool(i)            => *i,
            Dependency::Incompatible(i)    => *i,
            Dependency::Include(i)         => *i,
        }
    }
    pub fn idx(&self) -> u64 {
        match self {
            Dependency::EmbeddedLibrary(_) => 1,
            Dependency::Optional(_)        => 2,
            Dependency::Required(_)        => 3,
            Dependency::Tool(_)            => 4,
            Dependency::Incompatible(_)    => 5,
            Dependency::Include(_)         => 6,
        }
    }
    
    pub fn from_idx(idx: u64, id: AddonID) -> Option<Self> {
        match idx {
            1 => Some(Self::EmbeddedLibrary(id)),
            2 => Some(Self::Optional(id)),
            3 => Some(Self::Required(id)),
            4 => Some(Self::Tool(id)),
            5 => Some(Self::Incompatible(id)),
            6 => Some(Self::Include(id)),
            _ => None,
        }
    }
}

impl From<FileDependency> for Dependency {
    fn from(dep: FileDependency) -> Self {
        let id = AddonID(dep.mod_id.try_into().unwrap());
        match dep.relation_type {
            FileRelationType::EmbeddedLibrary => Self::EmbeddedLibrary(id),
            FileRelationType::OptionalDependency => Self::Optional(id),
            FileRelationType::RequiredDependency => Self::Required(id),
            FileRelationType::Tool => Self::Tool(id),
            FileRelationType::Incompatible => Self::Incompatible(id),
            FileRelationType::Include => Self::Include(id),
        }
    }
}

#[derive(Deserialize,Serialize,Clone)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Dependencies(Vec<Dependency>);

impl Dependencies {
    pub fn iter_embedded_library(&self) -> impl Iterator<Item=AddonID>+'_ {
        self.iter()
            .filter(|d| matches!(d,Dependency::EmbeddedLibrary(_)) )
            .map(|d| d.id() )
    }
    pub fn iter_optional(&self) -> impl Iterator<Item=AddonID>+'_ {
        self.iter()
            .filter(|d| matches!(d,Dependency::Optional(_)) )
            .map(|d| d.id() )
    }
    pub fn iter_required(&self) -> impl Iterator<Item=AddonID>+'_ {
        self.iter()
            .filter(|d| matches!(d,Dependency::Required(_)) )
            .map(|d| d.id() )
    }
    pub fn iter_tool(&self) -> impl Iterator<Item=AddonID>+'_ {
        self.iter()
            .filter(|d| matches!(d,Dependency::Tool(_)) )
            .map(|d| d.id() )
    }
    pub fn iter_incompatible(&self) -> impl Iterator<Item=AddonID>+'_ {
        self.iter()
            .filter(|d| matches!(d,Dependency::Incompatible(_)) )
            .map(|d| d.id() )
    }
    pub fn iter_include(&self) -> impl Iterator<Item=AddonID>+'_ {
        self.iter()
            .filter(|d| matches!(d,Dependency::Include(_)) )
            .map(|d| d.id() )
    }

    pub fn new_required(&self, new: &Dependencies) -> bool {
        for r in new.iter_required() {
            if !self.0.contains(&Dependency::Required(r)) {
                return true;
            }
        }
        false
    }
}

impl Deref for Dependencies {
    type Target=Vec<Dependency>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Dependencies {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<FileDependency>> for Dependencies {
    fn from(deps: Vec<FileDependency>) -> Self {
        Self(deps.into_iter().map(Into::into).collect())
    }
}

#[derive(Deserialize,Serialize)]
#[serde(rename_all="camelCase")]
struct DepIntermediate {
    addon_id: AddonID,
    r#type: u64,
}

impl Serialize for Dependency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        
            DepIntermediate{
            addon_id: self.id(),
            r#type: self.idx(),
        }
        .serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for Dependency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        DepIntermediate::deserialize(deserializer)
        .map(|dep| 
            Self::from_idx(dep.r#type,dep.addon_id)
                .unwrap_or_else(|| panic!("Unknown DependencyType {}",dep.r#type))
        )
    }
}
