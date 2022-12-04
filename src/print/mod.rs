use std::fmt::{Display, Debug};
use std::ops::Add;

use termion::terminal_size;

use crate::Op;
use crate::addon::files::AddonFile;
use crate::addon::release_type::ReleaseType;

pub mod versions;
pub mod addons;
pub mod error;

pub fn color_of_release_type(rt: &ReleaseType) -> Koller {
    match rt {
        ReleaseType::Alpha   => Koller::red(),
        ReleaseType::Beta    => Koller::yellow(),
        ReleaseType::Release => Koller::green(),
    }
}
pub fn color_of_release_type_bold(rt: &ReleaseType) -> Koller {
    match rt {
        ReleaseType::Alpha   => Koller::red_bold(),
        ReleaseType::Beta    => Koller::yellow_bold(),
        ReleaseType::Release => Koller::green_bold(),
    }
}
pub fn release_type_prefix(rt: &ReleaseType) -> &'static str {
    match rt {
        ReleaseType::Alpha   => "ALPHA:     ",
        ReleaseType::Beta    => "BETA:      ",
        ReleaseType::Release => "RELEASE:   ",
    }
}
pub fn release_type_str(rt: &ReleaseType) -> &'static str {
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
        format!("{d} ({f})")
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

#[derive(Clone,Copy)]
pub struct Koller {
    pub a: &'static str,
    pub b: &'static str,
    pub c: &'static str,
    pub d: &'static str,
}

impl Koller {
    pub fn red_bold() -> Self {
        use termion::color::*;
        use termion::style as S;
        Self {
            a: LightRed.fg_str(),
            b: S::Bold.as_ref(),
            c: S::Reset.as_ref(),
            d: Reset.fg_str(),
        }
    }
    pub fn yellow_bold() -> Self {
        use termion::color::*;
        use termion::style as S;
        Self {
            a: LightYellow.fg_str(),
            b: S::Bold.as_ref(),
            c: S::Reset.as_ref(),
            d: Reset.fg_str(),
        }
    }
    pub fn green_bold() -> Self {
        use termion::color::*;
        use termion::style as S;
        Self {
            a: LightGreen.fg_str(),
            b: S::Bold.as_ref(),
            c: S::Reset.as_ref(),
            d: Reset.fg_str(),
        }
    }
    pub fn blue_bold() -> Self {
        use termion::color::*;
        use termion::style as S;
        Self {
            a: LightBlue.fg_str(),
            b: S::Bold.as_ref(),
            c: S::Reset.as_ref(),
            d: Reset.fg_str(),
        }
    }
    pub fn red() -> Self {
        use termion::color::*;
        Self {
            a: LightRed.fg_str(),
            b: "",
            c: "",
            d: Reset.fg_str(),
        }
    }
    pub fn yellow() -> Self {
        use termion::color::*;
        Self {
            a: LightYellow.fg_str(),
            b: "",
            c: "",
            d: Reset.fg_str(),
        }
    }
    pub fn green() -> Self {
        use termion::color::*;
        Self {
            a: LightGreen.fg_str(),
            b: "",
            c: "",
            d: Reset.fg_str(),
        }
    }
    pub fn blue() -> Self {
        use termion::color::*;
        Self {
            a: LightBlue.fg_str(),
            b: "",
            c: "",
            d: Reset.fg_str(),
        }
    }
}

impl Default for Koller {
    fn default() -> Self {
        Self {
            a: "",
            b: "",
            c: "",
            d: "",
        }
    }
}

impl<T> Add<T> for Koller {
    type Output = KollerApplied<T>;

    fn add(self, rhs: T) -> Self::Output {
        KollerApplied {
            koller: self,
            value: rhs,
        }
    }
}

#[derive(Clone,Copy)]
pub struct KollerApplied<T> where T: ?Sized {
    koller: Koller,
    value: T,
}

impl<T> Display for KollerApplied<T> where T: Display + ?Sized {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.koller.a)?;
        f.write_str(self.koller.b)?;
        Display::fmt(&self.value, f)?;
        f.write_str(self.koller.c)?;
        f.write_str(self.koller.d)?;
        Ok(())
    }
}

impl<T> Debug for KollerApplied<T> where T: Debug + ?Sized {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.koller.a)?;
        f.write_str(self.koller.b)?;
        Debug::fmt(&self.value, f)?;
        f.write_str(self.koller.c)?;
        f.write_str(self.koller.d)?;
        Ok(())
    }
}
