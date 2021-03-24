use crate::conf::Repo;
use crate::{Op, error, hard_error};
use crate::util::match_str::match_str;
use super::match_bool;
use crate::unwrap_result_error;

pub fn main(
    o: &Op,
    repo: &mut Repo,
    key: Option<String>,
    value: Option<String>,
) -> bool {
    if let Some(key) = key {
        match match_key(&key) {
            WhatRSet::UrlTxt => if let Some(value) = value {
                if o.noop {return false;}
                repo.conf.url_txt = match_bool(&value,"url-txt");
                true
            } else {
                eprintln!("\turl-txt={}",repo.conf.url_txt);
                false
            },
            WhatRSet::AddonMtime => if let Some(value) = value {
                if o.noop {return false;}
                repo.conf.addon_mtime = match_bool(&value,"addon-mtime");
                true
            } else {
                eprintln!("\taddon-mtime={}",repo.conf.addon_mtime);
                false
            },
            WhatRSet::SoftRetries => if let Some(value) = value {
                if o.noop {return false;}
                repo.conf.soft_retries = unwrap_result_error!(value.trim().parse::<usize>());
                true
            } else {
                eprintln!("\tsoft-retries={}",repo.conf.soft_retries);
                false
            },
        }
    }else{
        eprintln!(
            "\turl-txt={}\n\taddon-mtime={}\n\tsoft-retries={}",
            repo.conf.url_txt,
            repo.conf.addon_mtime,
            repo.conf.soft_retries,
        );
        false
    }
}

fn match_key(s: &str) -> WhatRSet {
    let to_match = &[&[
        (WhatRSet::UrlTxt,"url-txt"),
        (WhatRSet::AddonMtime,"addon-mtime"),
        (WhatRSet::SoftRetries,"soft-retries"),
    ][..]][..];
    match match_str(s,to_match) {
        Ok(r) => r.z,
        Err(e) if e.is_empty() => hard_error!("Not match for setting"),
        Err(e) => {
            error!("Ambiguous matches for setting");
            for m in e {
                m.print_error();
            }
            std::process::exit(1);
        }
    }
}

#[derive(Clone)]
enum WhatRSet {
    UrlTxt,
    AddonMtime,
    SoftRetries,
}
