use crate::addon::release_type::ReleaseType;

pub mod aset;

pub fn release_type_from_flags(a: bool, b: bool, r: bool) -> Option<ReleaseType> {
    if a {
        Some(ReleaseType::Alpha)
    } else if b {
        Some(ReleaseType::Beta)
    } else if r {
        Some(ReleaseType::Release)
    } else {
        None
    }
}
