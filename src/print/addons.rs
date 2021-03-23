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
                installed.get(&a.id).and_then(|a| a.installed.as_ref() ).map(|a| a.release_type ),
                term_w() as usize,
                Default::default(),
            );
        } else {
            print_addon(
                &a.slug,
                &a.name,
                &a.summary,
                None,
                None,
                term_w() as usize,
                Koller::red_bold(),
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
                Default::default(),
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
    color: Koller,
) {
    let prefix = format!("{}: {}",slug,name);

    let mut suffix1 = "".to_owned();
    if let Some(rt) = avail_rt {
        let c = color_of_release_type(&rt);
        suffix1 = format!(" {}{}{}{}{}",c.a,c.b,release_type_prefix(&rt),c.c,c.d);
    }

    let mut suffix2 = "".to_owned();
    if let Some(rt) = installed_rt {
        let c = color_of_release_type(&rt);
        suffix2 = format!(" @{}{}{}{}{}",c.a,c.b,release_type_prefix(&rt),c.c,c.d);
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

    eprintln!(
        "{}{}{}{}{}{}{}{}",
        color.a,color.b,
        prefix,summary,
        color.c,color.d,
        suffix1,suffix2
    );
}
