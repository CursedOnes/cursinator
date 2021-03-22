use crate::addon::release_type::ReleaseType;
use crate::{error,hard_error};
use crate::util::match_str::match_str;

pub mod aset;
//pub mod gset;

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

fn match_bool(s: &str, caption: &str) -> bool {
    let to_match = &[
        (false,"false"),
        (true,"true"),
        (false,"no"),
        (true,"yes"),
        (false,"0"),
        (true,"1"),
    ];
    match match_str(s,||to_match.iter().cloned()) {
        Ok(r) => r.z,
        Err(e) if e.is_empty() => hard_error!("{} must be true/false/yes/no/0/1",caption),
        Err(e) => {
            error!("Ambiguous matches for {}",caption);
            for m in e {
                error!("\t{}{}{}{}{}",m.prefix(),Fg(Blue),m.marked(),Fg(Reset),m.suffix());
            }
            std::process::exit(1);
        }
    }
}
