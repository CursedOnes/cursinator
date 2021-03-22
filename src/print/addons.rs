use std::usize;

use termion::color::*;
use termion::terminal_size;

use crate::addon::local::{LocalAddon, LocalAddons};
use crate::addon::release_type::ReleaseType;
use crate::addon::{AddonSlug, GameVersion};
use crate::api::AddonInfo;
use super::*;

pub fn print_addons_search(
    addons: &[AddonInfo],
    game_version: &GameVersion,
    installed: &LocalAddons,
){
    for a in addons {
        if let Some(release_type) = a.release_type(game_version) {
            print_addon(
                &a.slug,
                &a.name,
                &a.summary,
                Some(release_type),
                todo!(),
                term_w() as usize,
                Reset.fg_str()
            );
        } else {
            print_addon(
                &a.slug,
                &a.name,
                &a.summary,
                None,
                None,
                term_w() as usize,
                Red.fg_str()
            );
        }
    }
}

pub fn print_addons_local(
    installed: &LocalAddons,
){
    for addon in installed.values()  {
        if let Some(addon_file) = &addon.installed {
            print_addon(
                &addon.slug,
                &addon.name,
                "",
                None,
                Some(addon_file.release_type),
                term_w() as usize,
                Reset.fg_str()
            );
        }
    }
}

pub fn print_addon(
    slug: &AddonSlug,
    name: &str,
    summary: &str,
    avail_rt: Option<ReleaseType>,
    installed_rt: Option<ReleaseType>,
    max_width: usize,
    color: &str,
) {
    let prefix = format!("{}: {}",slug.0,name);

    let mut suffix1 = "".to_owned();
    if let Some(rt) = avail_rt {
        suffix1 = format!(" {}{}{}",color_of_release_type(&rt),release_type_prefix(&rt),Fg(Reset));
    }

    let mut suffix2 = "".to_owned();
    if let Some(rt) = installed_rt {
        suffix2 = format!(" @{}{}{}",color_of_release_type(&rt),release_type_prefix(&rt),Fg(Reset));
    }

    let summary_width = max_width - prefix.len() - suffix1.len() - suffix2.len();

    let mut summ = "".to_owned();
    if summary_width >= 12 {
        if summary_width < summary.len() {
            summ = format!(" {}...",&summary[..summary_width-4]);
        } else {
            summ = format!(" {}",summary);
        }
    }

    eprintln!("{}{}{}{}{}{}",color,prefix,summary,Fg(Reset),suffix1,suffix2);
}
