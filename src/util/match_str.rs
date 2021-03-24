use std::ops::Range;

use termion::style;

use crate::addon::{AddonID, GameVersion};
use crate::addon::files::AddonFile;
use crate::addon::local::LocalAddons;
use crate::print::Koller;

use super::*;

pub fn find_installed_mod_by_key(s: &str, v: &LocalAddons, purge_mode: bool) -> Result<Match<AddonID>,Vec<Match<AddonID>>> {
        let iter_slug = v.values()
            .filter(|v| v.installed.is_some() || purge_mode )
            .map(|v| (v.id,v.slug.0.trim().to_owned()) )
            .collect();
        let iter_name = v.values()
            .filter(|v| v.installed.is_some() || purge_mode )
            .map(|v| (v.id,v.name.trim().to_owned()) )
            .collect();
        let iter_filename = v.values() 
            .filter_map(|v| v.installed.as_ref().map(|w| (v.id,w) ) )
            .map(|(z,v)| {
                let v = v.file_name.trim();
                if let Some(i) = v.rfind('/') {
                    (z,v[i+1..].to_owned())
                }else{
                    (z,v[..].to_owned())
                }
            })
            .collect();

    match_str::match_str(s,vec![iter_slug,iter_name,iter_filename])
}
pub fn find_to_install_version_by_key<'a>(s: &str, v: &'a [AddonFile], game_version: &GameVersion) -> Result<Match<&'a AddonFile>,Vec<Match<&'a AddonFile>>> {
        let iter_display_name = v.iter()
            .filter(|v| game_version.matches(v.game_version.iter()) )
            .map(|v| (v,v.display_name.trim().to_owned()) )
            .collect();
        let iter_file_name = v.iter()
            .filter(|v| game_version.matches(v.game_version.iter()) )
            .map(|v| (v,v.file_name.trim().to_owned()) )
            .collect();

    match_str::match_str(s,vec![iter_display_name,iter_file_name])
}

pub fn match_str<'a,Z>(s: &str, mut srcs: Vec<Vec<(Z,String)>>) -> Result<Match<Z>,Vec<Match<Z>>> where Z: Clone {
    fn match_in<'a,Z>(s: &str, src: &[(Z,String)]) -> Vec<Match<Z>> where Z: Clone {
        src.iter()
            .filter_map(|(z,v)| {
                v.trim().find(s).map(|off| Match {
                    z: z.clone(),
                    string: v.to_owned(),
                    range: off..off+s.len()
                })
            })
            .collect()
    }
    // 1. lowercase
    // 2. _-\ removed
    // 3. no spaces
    let s = s.trim();

    for src in &srcs[..] {
        let mut matches = match_in(s,src);
        if matches.len() == 1 {return Ok(matches.swap_remove(0));}
        if matches.len() > 1 {return Err(matches);}
    }

    let s = s.to_ascii_lowercase();

    for src in &mut srcs[..] {
        for s in src.iter_mut() {s.1.make_ascii_lowercase();}

        let mut matches = match_in(&s,src);
        if matches.len() == 1 {return Ok(matches.swap_remove(0));}
        if matches.len() > 1 {return Err(matches);}
    }

    let s = s.replace(' ',"_").replace('-',"_");

    for src in &mut srcs[..] {
        for s in src.iter_mut() {s.1 = s.1.replace(' ',"_").replace('-',"_");}

        let mut matches = match_in(&s,src);
        if matches.len() == 1 {return Ok(matches.swap_remove(0));}
        if matches.len() > 1 {return Err(matches);}
    }

    let s = s.replace('_',"");

    for src in &mut srcs[..] {
        for s in src.iter_mut() {s.1 = s.1.replace('_',"");}

        let mut matches = match_in(&s,src);
        if matches.len() == 1 {return Ok(matches.swap_remove(0));}
        if matches.len() > 1 {return Err(matches);}
    }

    Err(vec![])
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
