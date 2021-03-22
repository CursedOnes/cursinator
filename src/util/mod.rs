
pub mod match_str;

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
                error!("{}",e);
                None
            },
        }
    };
    ($oof:expr, |$e: ident| $($arg:tt)* ) => {
        match $oof {
            Ok(v) => Some(v),
            Err(e) => {
                let $e = e;
                error!($($arg)*);
                None
            },
        }
    };
    ($oof:expr, $($arg:tt)* ) => {
        match $oof {
            Ok(v) => Some(v),
            Err(_) => {
                error!($($arg)*);
                None
            },
        }
    };
}
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        use termion::color::*; 
        eprintln!("{}Error: {}{}",Fg(Red),Fg(Reset),format!($($arg)*));
    }}
}
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        use termion::color::*; 
        eprintln!("{}Warn: {}{}",Fg(Yellow),Fg(Reset),format!($($arg)*));
    }}
}
