use std::ffi::{OsStr, OsString};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};


pub mod match_str;

pub fn part_file_path(p: impl AsRef<OsStr>) -> PathBuf {
    let mut s: OsString = p.as_ref().to_owned();
    s.push(".part");
    PathBuf::from(s)
}


pub fn remove_if(path: impl AsRef<Path>) -> std::io::Result<bool> {
    match std::fs::remove_file(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e)
    }
}

#[macro_export]
macro_rules! hard_assert {
    ($oof:expr,$($arg:tt)*) => {{
        if !$oof {
            $crate::error!($($arg)*);
            std::process::exit(1);
        }
    }}
}

#[macro_export]
macro_rules! hard_error {
    ($($arg:tt)*) => {{
        $crate::error!($($arg)*);
        std::process::exit(1);
    }}
}

#[macro_export]
macro_rules! log_error {
    ($oof:expr) => {
        match $oof {
            Ok(v) => Some(v),
            Err(e) => {
                $crate::error!("{}",e);
                None
            },
        }
    };
    ($oof:expr, |$e: ident| $($arg:tt)* ) => {
        match $oof {
            Ok(v) => Some(v),
            Err(e) => {
                let $e = e;
                $crate::error!($($arg)*);
                None
            },
        }
    };
    ($oof:expr, $($arg:tt)* ) => {
        match $oof {
            Ok(v) => Some(v),
            Err(_) => {
                $crate::error!($($arg)*);
                None
            },
        }
    };
}
#[macro_export]
macro_rules! unwrap_result_error {
    ($oof:expr) => {
        match $oof {
            Ok(v) => v,
            Err(e) => $crate::hard_error!("{}",e),
        }
    };
    ($oof:expr, |$e: ident| $($arg:tt)* ) => {
        match $oof {
            Ok(v) => v,
            Err(e) => {
                let $e = e;
                $crate::hard_error!($($arg)*)
            },
        }
    };
    ($oof:expr, $($arg:tt)* ) => {
        match $oof {
            Ok(v) => v,
            Err(_) => $crate::hard_error!($($arg)*),
        }
    };
}
#[macro_export]
macro_rules! unwrap_or_error {
    ($oof:expr, $($arg:tt)* ) => {
        match $oof {
            Some(v) => v,
            None => $crate::hard_error!($($arg)*),
        }
    };
}
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        use termion::color::*;
        use termion::style::Bold;
        use termion::style::Reset as SReset;
        eprintln!("{}{}error: {}{}{}",Fg(LightRed),Bold,SReset,Fg(Reset),format!($($arg)*));
    }}
}
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        use termion::color::*;
        use termion::style::Bold;
        use termion::style::Reset as SReset;
        eprintln!("{}{}warn: {}{}{}",Fg(LightYellow),Bold,SReset,Fg(Reset),format!($($arg)*));
    }}
}
