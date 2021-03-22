use std::borrow::Borrow;

use termion::color::*;

use crate::addon::{AddonSlug, GameVersion};
use crate::util::match_str::Match;

use super::*;

pub fn print_versions(
    versions: &[impl AsAddonFile],
    current: Option<&AddonFile>,
    override_current_release_type: Option<ReleaseType>,
    fallback_release_type: ReleaseType,
    //current_slug: &AddonSlug,
    game_version: &GameVersion,
    print_older: bool,
    show_all: bool,
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

    let mut visible: Vec<Option<&AddonFile>> = Vec::with_capacity(versions.len()-current_idx);
    let mut visible_old: Vec<Option<&AddonFile>> = Vec::with_capacity(current_idx); //TODO no alloc if !print_older

    let target_release_type = override_current_release_type
        .or(current.map(|c| c.release_type))
        .unwrap_or(fallback_release_type);

    push_visible(
        &mut visible,
        target_release_type,
        show_all,
        game_version,
        versions.iter().map(AsAddonFile::file)
    );
    if print_older {
        push_visible(
            &mut visible_old,
            target_release_type,
            show_all,
            game_version,
            versions.iter().rev().map(AsAddonFile::file)
        );
    } else if current_idx != 0 {
        push_none(&mut visible_old);
    }
    
    for f in visible.iter().rev().chain(visible_old.iter()) {
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
    push_all: bool,
    game_version: &GameVersion,
    versions: impl Iterator<Item=&'a AddonFile>,
){
    for f in versions {
        if game_version.matches(f.game_version.iter()) {
            if f.release_type >= initial || push_all {
                initial = f.release_type;
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
        format!("{}{}{}{}{}",self.prefix(),Fg(Blue),self.marked(),Fg(Reset),self.suffix())
    }
}
