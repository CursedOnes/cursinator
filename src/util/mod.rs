pub mod match_str;
pub mod fs;

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
        let c = $crate::print::Koller::red_bold();
        eprintln!("{}{}error: {}{}{}",c.a,c.b,c.c,c.d,format!($($arg)*));
    }}
}
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        let c = $crate::print::Koller::yellow_bold();
        eprintln!("{}{}warn: {}{}{}",c.a,c.b,c.c,c.d,format!($($arg)*));
    }}
}
#[macro_export]
macro_rules! dark_log {
    ($($arg:tt)*) => {{
        use termion::color::*;
        let x = format!($($arg)*);
        eprintln!("{}{}{}",Fg(Rgb(127,127,127)),x,Fg(Reset));
    }}
}
