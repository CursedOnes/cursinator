use std::ops::{Deref, DerefMut};

use super::*;

#[derive(Clone)]
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

#[derive(Deserialize,Serialize,Clone)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Dependencies(Vec<Dependency>);

impl Dependencies {
    pub fn iter_embedded_library<'a>(&'a self) -> impl Iterator<Item=AddonID>+'a {
        self.iter()
            .filter(|d| matches!(d,Dependency::EmbeddedLibrary(_)) )
            .map(|d| d.id() )
    }
    pub fn iter_optional<'a>(&'a self) -> impl Iterator<Item=AddonID>+'a {
        self.iter()
            .filter(|d| matches!(d,Dependency::Optional(_)) )
            .map(|d| d.id() )
    }
    pub fn iter_required<'a>(&'a self) -> impl Iterator<Item=AddonID>+'a {
        self.iter()
            .filter(|d| matches!(d,Dependency::Required(_)) )
            .map(|d| d.id() )
    }
    pub fn iter_tool<'a>(&'a self) -> impl Iterator<Item=AddonID>+'a {
        self.iter()
            .filter(|d| matches!(d,Dependency::Tool(_)) )
            .map(|d| d.id() )
    }
    pub fn iter_incompatible<'a>(&'a self) -> impl Iterator<Item=AddonID>+'a {
        self.iter()
            .filter(|d| matches!(d,Dependency::Incompatible(_)) )
            .map(|d| d.id() )
    }
    pub fn iter_include<'a>(&'a self) -> impl Iterator<Item=AddonID>+'a {
        self.iter()
            .filter(|d| matches!(d,Dependency::Include(_)) )
            .map(|d| d.id() )
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
                .expect(&format!("Unknown DependencyType {}",dep.r#type))
        )
    }
}

/*pub struct Dependencies {
    pub embedded_library: Vec<AddonID>,
    pub optional:         Vec<AddonID>,
    pub required:         Vec<AddonID>,
    pub tool:             Vec<AddonID>,
    pub incompatible:     Vec<AddonID>,
    pub include:          Vec<AddonID>,
}

impl Serialize for Dependencies {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {

        let mut dest = Vec::<Dependency>::new();
        for &i in &self.embedded_library {
            dest.push(Dependency::EmbeddedLibrary(i));
        }
        for &i in &self.optional {
            dest.push(Dependency::Optional(i));
        }
        for &i in &self.required {
            dest.push(Dependency::Required(i));
        }
        for &i in &self.tool {
            dest.push(Dependency::Tool(i));
        }
        for &i in &self.incompatible {
            dest.push(Dependency::Incompatible(i));
        }
        for &i in &self.include {
            dest.push(Dependency::Include(i));
        }
        dest.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for Dependencies {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
       
        let intermediate = Vec::<Dependency>::deserialize(deserializer)?;

        let mut embedded_library = Vec::<AddonID>::with_capacity(intermediate.len());
        let mut optional         = Vec::<AddonID>::with_capacity(intermediate.len());
        let mut required         = Vec::<AddonID>::with_capacity(intermediate.len());
        let mut tool             = Vec::<AddonID>::with_capacity(intermediate.len());
        let mut incompatible     = Vec::<AddonID>::with_capacity(intermediate.len());
        let mut include          = Vec::<AddonID>::with_capacity(intermediate.len());

        for i in intermediate {
            match i {
                Dependency::EmbeddedLibrary(id) => embedded_library.push(id),
                Dependency::Optional(id) => optional.push(id),
                Dependency::Required(id) => required.push(id),
                Dependency::Tool(id) => tool.push(id),
                Dependency::Incompatible(id) => incompatible.push(id),
                Dependency::Include(id) => include.push(id),
            }
        }

        embedded_library.shrink_to_fit();
        optional.shrink_to_fit();
        required.shrink_to_fit();
        tool.shrink_to_fit();
        incompatible.shrink_to_fit();
        include.shrink_to_fit();

        Ok(Self{
            embedded_library,
            optional,
            required,
            tool,
            incompatible,
            include,
        })
    }
}*/
