use crate::addon::rtm::ReleaseTypeMode;
use crate::hard_error;

pub fn decode_channel(s: &str) -> ReleaseTypeMode {
    let (mut a,mut b,mut r) = (false,false,false);

    for c in s.trim().chars() {
        match c {
            'a' | 'A' => a=true,
            'b' | 'B' => b=true,
            'r' | 'R' => r=true,
            _ => hard_error!("Channel must consist of the letters a/b/r (e.g. r / b / a / rb / rba / ba / ra)"),
        }
    }

    ReleaseTypeMode::new(r,b,a)
}
