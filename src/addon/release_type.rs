
#[derive(Copy,Clone,PartialEq,Eq)]
pub enum ReleaseType {
    Alpha,
    Beta,
    Release,
}

impl ReleaseType {
    pub fn from_number(i: u32) -> Self {
        match i {
            1 => Self::Release,
            2 => Self::Beta,
            3 => Self::Alpha,
            _ => panic!("Unknown ReleaseType {}",i),
        }
    }
    pub fn to_number(&self) -> u32 {
        match self {
            Self::Release => 1,
            Self::Beta => 2,
            Self::Alpha => 3,
        }
    }

    pub fn more_stable_than(&self, other: &Self) -> bool {
        self.to_number() <= other.to_number()
    }

    pub fn max(&self, other: &Self) -> Self {
        Self::from_number( self.to_number().min(other.to_number()) )
    }
}

impl PartialOrd for ReleaseType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.to_number().cmp(&other.to_number()).reverse())
    }
}
impl Ord for ReleaseType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl serde::Serialize for ReleaseType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        self.to_number().serialize(serializer)
    }
}
impl<'de> serde::Deserialize<'de> for ReleaseType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        u32::deserialize(deserializer)
            .map(Self::from_number)
    }
}
