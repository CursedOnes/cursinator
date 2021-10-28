use std::ops::Range;

use crate::addon::{AddonID, GameVersion};
use crate::addon::files::AddonFile;
use crate::addon::local::LocalAddons;
use crate::print::Koller;

use super::*;

pub fn find_installed_mod_by_key(s: &str, v: &LocalAddons, purge_mode: bool) -> Result<Match<AddonID>,Vec<Match<AddonID>>> {
        let iter_slug: Vec<_> = v.values()
            .filter(|v| v.installed.is_some() || purge_mode )
            .map(|v| (v.id,v.slug.0.trim()) )
            .collect();
        let iter_name: Vec<_> = v.values()
            .filter(|v| v.installed.is_some() || purge_mode )
            .map(|v| (v.id,v.name.trim()) )
            .collect();
        let iter_filename: Vec<_> = v.values() 
            .filter_map(|v| v.installed.as_ref().map(|w| (v.id,w) ) )
            .map(|(z,v)| (z,cut_after_slash(v.file_name.trim())) )
            .collect();

    match_str::match_str(s,&[&iter_slug,&iter_name,&iter_filename])
}
pub fn find_to_install_version_by_key<'a>(s: &str, v: &'a [AddonFile], game_version: &GameVersion) -> Result<Match<&'a AddonFile>,Vec<Match<&'a AddonFile>>> {
        let iter_display_name: Vec<_> = v.iter()
            .filter(|v| game_version.matches(v.game_version.iter()) )
            .map(|v| (v,v.display_name.trim()) )
            .collect();
        let iter_file_name: Vec<_> = v.iter()
            .filter(|v| game_version.matches(v.game_version.iter()) )
            .map(|v| (v,cut_after_slash(v.file_name.trim())) )
            .collect();

    match_str::match_str(s,&[&iter_display_name,&iter_file_name])
}

pub fn match_str<Z>(s: &str, srcs: &[&[(Z,&str)]]) -> Result<Match<Z>,Vec<Match<Z>>> where Z: Clone {
    fn match_sub<Z>(s: &str, srcs: &[&[(Z,&str)]], f: impl Fn(&(Z,String))->Option<Match<Z>>) -> Result<Match<Z>,Vec<Match<Z>>> where Z: Clone {
        fn match_in<Z>(s: &str, src: &[(Z,String)], f: impl Fn(&(Z,String))->Option<Match<Z>>) -> Vec<Match<Z>> where Z: Clone {
            src.iter()
                .filter_map(f)
                .collect()
        }
        // 1. lowercase
        // 2. _-\ removed
        // 3. no spaces
        let s = s.trim();

        let mut srcs: Vec<Vec<(Z,String)>> = srcs.iter()
            .map(|s| 
                s.iter()
                .map(|(z,s)| (z.clone(),s.trim().to_owned()) )
                .collect()
            )
            .collect();

        for src in &srcs[..] {
            let mut matches = match_in(s,src,&f);
            if matches.len() == 1 {return Ok(matches.swap_remove(0));}
            if matches.len() > 1 {return Err(matches);}
        }

        let mut s = s.to_ascii_lowercase();

        for src in &mut srcs[..] {
            for s in src.iter_mut() {s.1.make_ascii_lowercase();}

            let mut matches = match_in(&s,src,&f);
            if matches.len() == 1 {return Ok(matches.swap_remove(0));}
            if matches.len() > 1 {return Err(matches);}
        }

        string_remove_1(&mut s);

        for src in &mut srcs[..] {
            for s in src.iter_mut() {string_remove_1(&mut s.1);}

            let mut matches = match_in(&s,src,&f);
            if matches.len() == 1 {return Ok(matches.swap_remove(0));}
            if matches.len() > 1 {return Err(matches);}
        }

        string_remove_2(&mut s);

        for src in &mut srcs[..] {
            for s in src.iter_mut() {string_remove_2(&mut s.1);}

            let mut matches = match_in(&s,src,&f);
            if matches.len() == 1 {return Ok(matches.swap_remove(0));}
            if matches.len() > 1 {return Err(matches);}
        }

        string_remove_3(&mut s);

        for src in &mut srcs[..] {
            for s in src.iter_mut() {string_remove_3(&mut s.1);}

            let mut matches = match_in(&s,src,&f);
            if matches.len() == 1 {return Ok(matches.swap_remove(0));}
            if matches.len() > 1 {return Err(matches);}
        }

        Err(vec![])
    }

    let matches = match_sub(s, srcs, |(z,v)| {
        (v==s).then(||{
            Match {
                z: z.clone(),
                string: v.to_owned(),
                range: 0..s.len()
            }
        })
    });

    match matches {
        Ok(v) => return Ok(v),
        Err(v) if v.is_empty() => {},
        Err(v) => return Err(v),
    }

    match_sub(s, srcs, |(z,v)| {
        v.find(s).map(|off| Match {
            z: z.clone(),
            string: v.to_owned(),
            range: off..off+s.len()
        })
    })
}

fn cut_after_slash(s: &str) -> &str {
    if let Some(i) = s.rfind('/') {
        &s[i+1..]
    }else{
        s
    }
}

fn string_remove_1(s: &mut String) {
    unsafe{
        s.as_mut_vec()
            .retain(u8::is_ascii);
    }
    debug_assert!(std::str::from_utf8(s.as_bytes()).is_ok());
}

fn string_remove_2(s: &mut String) {
    for b in unsafe{s.as_bytes_mut()} {
        if *b == b' ' || *b == b'-' || *b == b',' {
            *b = b'_';
        }
    }
    debug_assert!(std::str::from_utf8(s.as_bytes()).is_ok());
}

fn string_remove_3(s: &mut String) {
    unsafe{
        s.as_mut_vec()
            .retain(|b| *b != b'_');
    }
    debug_assert!(std::str::from_utf8(s.as_bytes()).is_ok());
}

#[derive(Clone)]
pub struct Match<Z> {
    pub z: Z,
    pub string: String,
    pub range: Range<usize>,
}

impl<Z> Match<Z> {
    pub fn prefix(&self) -> &str {
        &self.string[..self.range.start]
    }
    pub fn marked(&self) -> &str {
        &self.string[self.range.clone()]
    }
    pub fn suffix(&self) -> &str {
        &self.string[self.range.end..]
    }
    pub fn print_error(&self) {
        let c = Koller::blue_bold();
        crate::error!(
            "\t{}{}{}{}{}{}{}",
            self.prefix(),
            c.a,c.b,
            self.marked(),
            c.c,c.d,
            self.suffix(),
        );
    }
}
