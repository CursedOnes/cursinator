use std::borrow::Borrow;

use termion::{color::*, style};
use termion::style::Bold;

use crate::addon::rtm::ReleaseTypeMode;
use crate::addon::{AddonSlug, GameVersion};
use crate::util::match_str::Match;

use super::*;

pub fn print_versions(
    versions: &[impl AsAddonFile],
    current: Option<&AddonFile>,
    release_type: ReleaseTypeMode,
    //current_slug: &AddonSlug,
    game_version: &GameVersion,
    print_older: bool,
    max_h: usize,
){
    let mut current_idx = 0;
    if let Some(current) = current {
        for v in versions {
            if v.file().id.0 < current.id.0 {
                current_idx += 1;
            } else {
                break
            }
        }
    } else {
        current_idx = versions.len();
    }

    let visible_range = if print_older || current.is_none() {
        0..versions.len()
    } else {
        current_idx..versions.len()
    };

    let mut visible: Vec<Option<&AddonFile>> = Vec::with_capacity(visible_range.len());

    let target_release_type = release_type.pick_level(
        versions[visible_range.clone()].iter().map(|f|f.file().release_type)
    );

    push_visible(
        &mut visible,
        target_release_type,
        current,
        true,
        game_version,
        versions[visible_range.clone()].iter().rev().map(AsAddonFile::file)
    );
    if visible_range.start != 0 {
        push_none(&mut visible);
    }

    if visible.len() > max_h {
        visible.clear();

        push_visible(
            &mut visible,
            target_release_type,
            current,
            false,
            game_version,
            versions[visible_range.clone()].iter().rev().map(AsAddonFile::file)
        );
        if visible_range.start != 0 {
            push_none(&mut visible);
        }
    }
    
    for f in visible.iter() {
        if let Some(f) = f {
            let color =
                if current.is_some() && f.id == current.unwrap().id {
                    Blue.fg_str()
                }else{
                    color_of_release_type(&f.release_type)
                };
            eprintln!("{}{}{}{}",
                color,release_type_prefix(&f.release_type),Fg(Reset),
                f.display(),
            );
        }else{
            eprintln!("...");
        }
    }
}

fn push_visible<'a>(
    dest: &mut Vec<Option<&'a AddonFile>>,
    mut initial: ReleaseType,
    current: Option<&AddonFile>,
    push_all: bool,
    game_version: &GameVersion,
    versions: impl Iterator<Item=&'a AddonFile>,
){
    for f in versions {
        if game_version.matches(f.game_version.iter()) {
            if f.release_type >= initial || push_all {
                initial = f.release_type;
                dest.push(Some(f));
            } else if current.is_some() && current.unwrap().id == f.id {
                dest.push(Some(f));
            } else {
                push_none(dest);
            }
        }
    }
}

fn push_none<T>(d: &mut Vec<Option<T>>) {
    if d[d.len()-1].is_some() {
        d.push(None);
    }
}

pub trait AsAddonFile {
    fn file(&self) -> &AddonFile;
    fn display(&self) -> String;
}

impl AsAddonFile for AddonFile {
    fn file(&self) -> &AddonFile {
        self
    }
    fn display(&self) -> String {
        addon_file_display_name(self)
    }
}

impl AsAddonFile for Match<&AddonFile> {
    fn file(&self) -> &AddonFile {
        self.z
    }
    fn display(&self) -> String {
        format!(
            "\t{}{}{}{}{}{}{}",
            self.prefix(),
            Bold,Fg(LightBlue),
            self.marked(),
            style::Reset,Fg(Reset),
            self.suffix(),
        )
    }
}
