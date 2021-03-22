use termion::terminal_size;

use crate::Op;
use crate::addon::files::AddonFile;
use crate::addon::release_type::ReleaseType;

pub mod versions;
pub mod addons;
pub mod error;

pub fn color_of_release_type(rt: &ReleaseType) -> &'static str {
    use termion::color::*;
    match rt {
        ReleaseType::Alpha   => LightRed.fg_str(),
        ReleaseType::Beta    => LightYellow.fg_str(),
        ReleaseType::Release => LightGreen.fg_str(),
    }
}
pub fn release_type_prefix(rt: &ReleaseType) -> &'static str {
    match rt {
        ReleaseType::Alpha   => "ALPHA  ",
        ReleaseType::Beta    => "BETA   ",
        ReleaseType::Release => "RELEASE",
    }
}

pub fn addon_file_display_name(f: &AddonFile) -> String {
    let d = f.display_name.trim();
    let f = f.file_name.trim();
    if d == f {
        f.to_owned()
    }else{
        format!("{} ({})",d,f)
    }
}

pub fn term_size() -> (u16,u16) {
    terminal_size().unwrap_or((16384,16384))
}
pub fn term_w() -> u16 {
    term_size().0
}
pub fn term_h() -> u16 {
    term_size().1
}

pub fn show_all_thresh(n: usize) -> bool {
    n+4 <= (term_h() as usize)
}

impl Op {
    pub fn suffix(&self) -> String {
        use termion::color::*;
        match self.noop {
            true => format!(" {}(noop){}",Fg(Rgb(127,127,127)),Fg(Reset)),
            false => "".to_owned()
        }
    }
}
